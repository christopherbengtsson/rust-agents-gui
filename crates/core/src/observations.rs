//! Bounded invalidation hints, independent of separately read feature records.
use crate::workspaces::{ActionId, RepositoryId, WorkspaceId};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    io,
    sync::{Condvar, Mutex},
    time::Duration,
};

pub const JOURNAL_ENTRIES: usize = 256;
pub const ENTRY_TAGS: usize = 8;
pub const DELIVERY_TAGS: usize = 32;
pub const DELIVERY_BYTES: usize = 4096;
pub const MAX_SEQUENCE: u64 = 9_007_199_254_740_991;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "snake_case")]
// Collection hints invalidate all related details and retained pending-action interests too.
// Narrow identity tags permit consumers to avoid unrelated work when identities are known.
pub enum Target {
    Repositories,
    Repository { repository_id: RepositoryId },
    Registration { action_id: ActionId },
    Workspaces { repository_id: Option<RepositoryId> },
    Workspace { workspace_id: WorkspaceId },
    WorkspaceAction { action_id: ActionId },
    Inspection { workspace_id: WorkspaceId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct FeedCursor {
    pub owner: String,
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Sample {
    Hints {
        watermark: u64,
        targets: Vec<Target>,
    },
    ResyncRequired {
        watermark: u64,
    },
    Heartbeat {
        watermark: u64,
    },
}
impl Sample {
    pub fn watermark(&self) -> u64 {
        match self {
            Self::Hints { watermark, .. }
            | Self::ResyncRequired { watermark }
            | Self::Heartbeat { watermark } => *watermark,
        }
    }
}

struct Entry {
    sequence: u64,
    targets: Vec<Target>,
}
struct State {
    sequence: u64,
    entries: VecDeque<Entry>,
}
pub struct Journal {
    state: Mutex<State>,
    changed: Condvar,
}
impl Default for Journal {
    fn default() -> Self {
        Self {
            state: Mutex::new(State {
                sequence: 1,
                entries: VecDeque::new(),
            }),
            changed: Condvar::new(),
        }
    }
}
impl Journal {
    pub fn watermark(&self) -> u64 {
        self.state.lock().expect("journal lock poisoned").sequence
    }
    /// Called under domain protection. The owning guard wakes only after unlocking.
    pub(crate) fn publish(&self, mut targets: Vec<Target>) {
        let mut state = self.state.lock().expect("journal lock poisoned");
        if targets.len() > ENTRY_TAGS {
            targets.clear();
        }
        if state.sequence == MAX_SEQUENCE {
            // No cursor aliasing: every subsequent observation must reconnect to a new owner.
            state.entries.clear();
            return;
        }
        state.sequence += 1;
        let sequence = state.sequence;
        if state.entries.len() == JOURNAL_ENTRIES {
            state.entries.pop_front();
        }
        state.entries.push_back(Entry { sequence, targets });
    }
    pub(crate) fn wake(&self) {
        self.changed.notify_all();
    }
    pub fn sample(&self, after: Option<u64>) -> Sample {
        Self::sample_state(&self.state.lock().expect("journal lock poisoned"), after)
    }
    fn sample_state(state: &State, after: Option<u64>) -> Sample {
        let watermark = state.sequence;
        let Some(after) = after.filter(|after| *after <= watermark && *after > 0) else {
            return Sample::ResyncRequired { watermark };
        };
        if watermark == MAX_SEQUENCE
            || (after < watermark
                && state
                    .entries
                    .front()
                    .is_none_or(|entry| after < entry.sequence - 1))
        {
            return Sample::ResyncRequired { watermark };
        }
        let mut targets = Vec::new();
        for entry in state.entries.iter().filter(|entry| entry.sequence > after) {
            if entry.targets.is_empty() {
                return Sample::ResyncRequired { watermark };
            }
            for target in &entry.targets {
                if !targets.contains(target) {
                    targets.push(target.clone());
                }
                if targets.len() > DELIVERY_TAGS {
                    return Sample::ResyncRequired { watermark };
                }
            }
        }
        if targets.is_empty() {
            return Sample::Heartbeat { watermark };
        }
        let sample = Sample::Hints { watermark, targets };
        if serde_json::to_vec(&sample).map_or(true, |bytes| bytes.len() > DELIVERY_BYTES - 512) {
            Sample::ResyncRequired { watermark }
        } else {
            sample
        }
    }
    /// Predicate and wait share one mutex: publication between sample and wait is observed.
    pub(crate) fn wait(&self, after: Option<u64>, timeout: Duration) -> io::Result<Sample> {
        let state = self
            .state
            .lock()
            .map_err(|e| io::Error::other(e.to_string()))?;
        let (state, _) = self
            .changed
            .wait_timeout_while(state, timeout, |state| {
                after == Some(state.sequence) && state.sequence != MAX_SEQUENCE
            })
            .map_err(|e| io::Error::other(e.to_string()))?;
        if state.sequence == MAX_SEQUENCE {
            return Err(io::Error::other(
                "Observation sequence exhausted; reconnect to a new supervisor owner",
            ));
        }
        Ok(Self::sample_state(&state, after))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn baseline_and_expired_or_future_cursors_require_resync() {
        let journal = Journal::default();
        assert!(matches!(
            journal.sample(None),
            Sample::ResyncRequired { .. }
        ));
        let baseline = journal.watermark();
        journal.publish(vec![Target::Repositories]);
        assert!(matches!(
            journal.sample(Some(baseline)),
            Sample::Hints { .. }
        ));
        for _ in 0..257 {
            journal.publish(vec![Target::Repositories]);
        }
        assert!(matches!(
            journal.sample(Some(baseline)),
            Sample::ResyncRequired { .. }
        ));
        assert!(matches!(
            journal.sample(Some(journal.watermark() + 1)),
            Sample::ResyncRequired { .. }
        ));
    }
}

#[cfg(test)]
mod bounds_tests {
    use super::*;
    use uuid::Uuid;
    #[test]
    fn tag_overflow_resync_and_new_changes_survive_the_sampled_watermark() {
        let journal = Journal::default();
        let baseline = journal.watermark();
        for index in 0..33 {
            journal.publish(vec![Target::Workspace {
                workspace_id: WorkspaceId(Uuid::from_u128(index)),
            }]);
        }
        let Sample::ResyncRequired { watermark } = journal.sample(Some(baseline)) else {
            panic!()
        };
        journal.publish(vec![Target::Repositories]);
        assert_eq!(
            journal.sample(Some(watermark)),
            Sample::Hints {
                watermark: watermark + 1,
                targets: vec![Target::Repositories]
            }
        );
        assert_eq!(journal.state.lock().unwrap().entries.len(), 34);
        journal.publish(vec![Target::Repositories; 9]);
        assert!(matches!(
            journal.sample(Some(watermark)),
            Sample::ResyncRequired { .. }
        ));
    }
    #[test]
    fn batches_deduplicate_and_retention_has_an_exact_boundary() {
        let journal = Journal::default();
        for _ in 0..256 {
            journal.publish(vec![Target::Repositories; 8]);
        }
        assert_eq!(journal.state.lock().unwrap().entries.len(), 256);
        assert_eq!(
            journal.sample(Some(1)),
            Sample::Hints {
                watermark: 257,
                targets: vec![Target::Repositories]
            }
        );
        journal.publish(vec![Target::Repositories]);
        assert!(matches!(
            journal.sample(Some(1)),
            Sample::ResyncRequired { .. }
        ));
        assert!(matches!(journal.sample(Some(2)), Sample::Hints { .. }));
        assert!(serde_json::to_vec(&journal.sample(Some(2))).unwrap().len() < DELIVERY_BYTES);
    }
    #[test]
    fn publication_before_wait_cannot_be_lost_and_idle_does_not_advance_cursor() {
        let journal = Journal::default();
        assert_eq!(
            journal.wait(Some(1), Duration::ZERO).unwrap(),
            Sample::Heartbeat { watermark: 1 }
        );
        journal.publish(vec![Target::Repositories]);
        journal.wake();
        assert!(matches!(
            journal.wait(Some(1), Duration::ZERO).unwrap(),
            Sample::Hints { watermark: 2, .. }
        ));
    }
    #[test]
    fn exhausted_sequence_never_aliases_an_accepted_cursor() {
        let journal = Journal::default();
        journal.state.lock().unwrap().sequence = MAX_SEQUENCE;
        journal.publish(vec![Target::Repositories]);
        assert_eq!(
            journal.sample(Some(MAX_SEQUENCE)),
            Sample::ResyncRequired {
                watermark: MAX_SEQUENCE
            }
        );
    }
}

#[cfg(feature = "codegen")]
pub(crate) fn export_contracts(exports: &mut crate::codegen::Exports) {
    exports.add::<Target>();
    exports.add::<FeedCursor>();
    exports.add::<Sample>();
}
