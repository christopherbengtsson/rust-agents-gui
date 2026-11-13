use super::*;

fn target() -> Target {
    Target {
        owner: "owner".into(),
        runtime: Runtime::Codex,
    }
}
fn save(id: Uuid, executable: Option<&str>) -> RuntimeRequest {
    RuntimeRequest::Save {
        target: target(),
        action_id: id,
        revision: 1,
        executable: executable.map(str::to_owned),
    }
}

#[test]
fn saving_rejects_relative_or_oversized_paths_but_reset_is_valid() {
    assert!(validate_preference(Some("codex")).is_err());
    assert!(validate_preference(Some(&format!("/{}", "x".repeat(MAX_EXECUTABLE_BYTES)))).is_err());
    assert!(validate_preference(Some("/bin/codex\0bad")).is_err());
    assert!(validate_preference(Some("/app with spaces/codex")).is_ok());
    assert!(validate_preference(None).is_ok());
}

#[test]
fn admission_serializes_setup_and_fences_changed_owner_and_configuration() {
    let mut state = State::ready(target(), Configuration::default());
    let request = save(Uuid::new_v4(), Some("/bin/codex"));
    assert!(state.admit(&request).unwrap());
    assert!(!state.admit(&save(Uuid::new_v4(), None)).unwrap());
    assert!(!state.admit(&request).unwrap());
    let mut stale = request.clone();
    if let RuntimeRequest::Save { target, .. } = &mut stale {
        target.owner = "old-owner".into();
    }
    assert_eq!(
        state.admit(&stale).unwrap_err().code,
        ErrorCode::StaleTarget
    );
}

#[derive(Default)]
struct Memory {
    configuration: Configuration,
    actions: std::collections::BTreeMap<Uuid, Action>,
    blocked: bool,
    calls: Vec<&'static str>,
    fail_intent: bool,
    save_uncertain: bool,
    fail_finish: bool,
    reject_save: bool,
}
impl RuntimeStore for Memory {
    fn configuration(&mut self) -> Result<Configuration> {
        self.calls.push("read-configuration");
        Ok(self.configuration.clone())
    }
    fn action(&mut self, id: Uuid) -> Result<Option<Action>> {
        self.calls.push("read-action");
        Ok(self.actions.get(&id).cloned())
    }
    fn blocked(&mut self) -> Result<bool> {
        Ok(self.blocked)
    }
    fn intent(&mut self, a: &Action, _: &str) -> Result<()> {
        self.calls.push("intent");
        if self.fail_intent {
            return Err(RuntimeError::storage());
        }
        self.actions.insert(a.action_id, a.clone());
        if matches!(a.request, RuntimeRequest::Check { .. }) {
            self.blocked = true;
        }
        Ok(())
    }
    fn save(&mut self, a: &Action, c: &Configuration) -> Result<()> {
        self.calls.push("save");
        if self.reject_save {
            return Err(RuntimeError::storage());
        }
        self.configuration = c.clone();
        self.actions.insert(a.action_id, a.clone());
        if self.save_uncertain {
            Err(RuntimeError::new(
                ErrorCode::PersistenceUncertain,
                "Synthetic lost commit result",
            ))
        } else {
            Ok(())
        }
    }
    fn finish(&mut self, a: &Action, clean: bool) -> Result<()> {
        self.calls.push("finish");
        if self.fail_finish {
            return Err(RuntimeError::storage());
        }
        self.actions.insert(a.action_id, a.clone());
        self.blocked &= !clean;
        Ok(())
    }
}
fn check(id: Uuid) -> RuntimeRequest {
    RuntimeRequest::Check {
        target: target(),
        action_id: id,
        revision: 1,
    }
}
fn success(c: &Configuration, id: Uuid) -> Diagnostic {
    Diagnostic {
        check_id: id,
        revision: c.revision,
        timestamp_ms: 123,
        executable: Some("/fixture/codex".into()),
        version: Some(SUPPORTED_VERSION.into()),
        authentication: "Unknown".into(),
        provider: None,
        billing: "Unknown".into(),
        passed: true,
        error: None,
        cleanup_confirmed: true,
        exit_successful: Some(true),
    }
}

#[test]
fn saved_preferences_invalidate_readiness_without_starting_a_process() {
    let shared = Mutex::new(State::ready(target(), Configuration::default()));
    let mut store = Memory::default();
    shared.lock().unwrap().reply.diagnostic =
        Some(success(&Configuration::default(), Uuid::new_v4()));
    execute(
        &shared,
        &mut store,
        save(Uuid::from_u128(1), Some("/bin/codex")),
        "boot",
        |_, _, _| panic!("Save started Codex"),
    );
    let s = shared.lock().unwrap();
    assert_eq!(
        s.reply
            .configuration
            .as_ref()
            .unwrap()
            .executable
            .as_deref(),
        Some("/bin/codex")
    );
    assert_eq!(s.reply.configuration.as_ref().unwrap().revision, 2);
    assert!(s.reply.diagnostic.is_none());
    assert!(!s.reply.busy);
}
#[test]
fn intent_is_durable_before_effect_and_repeated_identity_never_replays() {
    let shared = Mutex::new(State::ready(target(), Configuration::default()));
    let mut store = Memory::default();
    let id = Uuid::from_u128(1);
    let mut effects = 0;
    execute(&shared, &mut store, check(id), "boot", |c, id, _| {
        effects += 1;
        success(c, id)
    });
    assert_eq!(store.calls.iter().position(|c| *c == "intent"), Some(2));
    execute(&shared, &mut store, check(id), "boot", |_, _, _| {
        panic!("duplicate check replayed")
    });
    assert_eq!(effects, 1);
    assert_eq!(
        shared.lock().unwrap().reply.active.as_ref().unwrap().phase,
        Phase::Completed
    );
}
#[test]
fn failed_intent_or_surviving_cleanup_never_launches_replacement() {
    for (fail_intent, blocked) in [(true, false), (false, true)] {
        let shared = Mutex::new(State::ready(target(), Configuration::default()));
        let mut store = Memory {
            fail_intent,
            blocked,
            ..Default::default()
        };
        execute(
            &shared,
            &mut store,
            check(Uuid::new_v4()),
            "boot",
            |_, _, _| panic!("unrecorded/conflicting effect"),
        );
        assert!(
            !shared
                .lock()
                .unwrap()
                .reply
                .active
                .as_ref()
                .unwrap()
                .phase
                .eq(&Phase::Completed)
        );
    }
}
#[test]
fn lost_save_commit_result_is_settled_by_authoritative_read_without_repeated_write() {
    let shared = Mutex::new(State::ready(target(), Configuration::default()));
    let mut store = Memory {
        save_uncertain: true,
        ..Default::default()
    };
    execute(
        &shared,
        &mut store,
        save(Uuid::new_v4(), Some("/new/codex")),
        "boot",
        |_, _, _| panic!(),
    );
    let s = shared.lock().unwrap();
    assert_eq!(
        s.reply
            .configuration
            .as_ref()
            .unwrap()
            .executable
            .as_deref(),
        Some("/new/codex")
    );
    assert_eq!(s.reply.active.as_ref().unwrap().phase, Phase::Completed);
    assert_eq!(store.calls.iter().filter(|c| **c == "save").count(), 1);
    assert!(s.reply.blocked.is_none());
}
#[test]
fn unconfirmed_cleanup_and_failed_final_persistence_keep_the_replacement_fence() {
    for failed_finish in [false, true] {
        let shared = Mutex::new(State::ready(target(), Configuration::default()));
        let mut store = Memory {
            fail_finish: failed_finish,
            ..Default::default()
        };
        execute(
            &shared,
            &mut store,
            check(Uuid::new_v4()),
            "boot",
            |c, id, _| {
                let mut d = success(c, id);
                if !failed_finish {
                    d.cleanup_confirmed = false;
                    d.passed = false;
                    d.error = Some(RuntimeError::cleanup());
                }
                d
            },
        );
        let mut s = shared.lock().unwrap();
        assert_eq!(
            s.reply.blocked.as_ref().unwrap().code,
            ErrorCode::CleanupUnconfirmed
        );
        assert!(s.admit(&save(Uuid::new_v4(), None)).unwrap());
        // Admission here queues identity resolution only. The owning worker
        // applies durable cleanup fencing before any new effect.
    }
}
#[test]
fn action_reads_and_restart_markers_do_not_execute_checks() {
    let shared = Mutex::new(State::ready(target(), Configuration::default()));
    let mut store = Memory::default();
    let id = Uuid::from_u128(8);
    store.actions.insert(
        id,
        Action {
            action_id: id,
            request: check(id),
            phase: Phase::Uncertain,
            error: Some(RuntimeError::cleanup()),
        },
    );
    store.blocked = true;
    execute(
        &shared,
        &mut store,
        RuntimeRequest::Action {
            target: target(),
            action_id: id,
        },
        "new-owner",
        |_, _, _| panic!("status started Codex"),
    );
    let s = shared.lock().unwrap();
    assert!(s.reply.lookup.as_ref().unwrap().complete);
    assert_eq!(
        s.reply
            .lookup
            .as_ref()
            .unwrap()
            .action
            .as_ref()
            .unwrap()
            .phase,
        Phase::Uncertain
    );
    assert!(!store.calls.contains(&"intent"));
}

#[test]
fn blocked_storage_reads_reenter_authoritative_storage_even_with_a_cached_action() {
    let mut state = State::ready(target(), Configuration::default());
    let id = Uuid::new_v4();
    state.remember(Action {
        action_id: id,
        request: save(id, None),
        phase: Phase::Uncertain,
        error: Some(RuntimeError::storage()),
    });
    state.reply.blocked = Some(RuntimeError::storage());
    state.reply.lookup = Some(ActionLookup {
        action_id: id,
        complete: true,
        action: None,
    });
    assert!(
        state
            .admit(&RuntimeRequest::Action {
                target: target(),
                action_id: id
            })
            .unwrap()
    );
    let shared = Mutex::new(state);
    let mut store = Memory::default();
    execute(
        &shared,
        &mut store,
        RuntimeRequest::Action {
            target: target(),
            action_id: id,
        },
        "boot",
        |_, _, _| panic!("read replayed work"),
    );
    assert!(shared.lock().unwrap().reply.blocked.is_none());
    assert!(store.calls.contains(&"read-configuration"));
}

#[test]
fn confirmed_save_rollback_preserves_definite_rejection_and_previous_preference() {
    let shared = Mutex::new(State::ready(target(), Configuration::default()));
    let mut store = Memory {
        reject_save: true,
        ..Default::default()
    };
    let id = Uuid::new_v4();
    execute(
        &shared,
        &mut store,
        save(id, Some("/new/codex")),
        "boot",
        |_, _, _| panic!(),
    );
    let s = shared.lock().unwrap();
    let action = s.reply.active.as_ref().unwrap();
    assert_eq!(action.phase, Phase::Failed);
    assert_eq!(action.error.as_ref().unwrap().code, ErrorCode::Storage);
    assert!(s.reply.configuration.as_ref().unwrap().executable.is_none());
    assert_eq!(store.actions[&id].phase, Phase::Failed);
}

#[test]
fn bounded_worker_shutdown_keeps_ownership_on_timeout_and_joins_only_finished_work() {
    struct Worker {
        time: u64,
        finishes: u64,
        retained: bool,
    }
    impl Completion for Worker {
        fn finished(&self) -> bool {
            self.time >= self.finishes
        }
        fn elapsed(&self) -> std::time::Duration {
            std::time::Duration::from_secs(self.time)
        }
        fn tick(&mut self) {
            self.time += 1;
        }
        fn join(&mut self) -> Result<()> {
            assert!(self.finished());
            self.retained = false;
            Ok(())
        }
    }
    let mut worker = Worker {
        time: 0,
        finishes: 99,
        retained: true,
    };
    assert!(await_worker(&mut worker).is_err());
    assert!(worker.retained);
    assert_eq!(worker.time, 4);
    worker.finishes = worker.time;
    assert!(await_worker(&mut worker).is_ok());
    assert!(!worker.retained);
}

#[test]
fn public_repeated_mutations_return_exact_cached_and_evicted_action_status() {
    let old_id = Uuid::from_u128(1);
    let old_request = check(old_id);
    let old = Action {
        action_id: old_id,
        request: old_request.clone(),
        phase: Phase::Completed,
        error: None,
    };
    let mut state = State::ready(
        target(),
        Configuration {
            revision: 2,
            executable: None,
        },
    );
    state.remember(old.clone());
    state.reply.active = Some(Action {
        action_id: Uuid::from_u128(2),
        request: save(Uuid::from_u128(2), None),
        phase: Phase::Completed,
        error: None,
    });
    let shared = Arc::new(Mutex::new(state));
    let (sender, receiver) = mpsc::sync_channel(1);
    let runtime = Runtimes {
        shared: shared.clone(),
        sender: Some(sender),
        worker: None,
        stop: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    };
    let reply = runtime.request(old_request.clone()).unwrap();
    assert_eq!(reply.lookup.unwrap().action.unwrap().action_id, old_id);
    assert!(receiver.try_recv().is_err());
    shared.lock().unwrap().actions.clear();
    shared.lock().unwrap().reply.blocked = Some(RuntimeError::cleanup());
    let reply = runtime.request(old_request.clone()).unwrap();
    assert!(!reply.lookup.unwrap().complete);
    let queued = receiver.try_recv().unwrap();
    assert_eq!(queued, old_request);
    let mut store = Memory {
        configuration: Configuration {
            revision: 2,
            executable: None,
        },
        blocked: true,
        ..Default::default()
    };
    store.actions.insert(old_id, old);
    execute(&shared, &mut store, queued, "boot", |_, _, _| {
        panic!("evicted repeated identity replayed")
    });
    let reply = runtime.request(old_request).unwrap();
    let lookup = reply.lookup.unwrap();
    assert!(lookup.complete);
    assert_eq!(lookup.action.unwrap().phase, Phase::Completed);
}

#[test]
fn evicted_conflicting_payload_is_rejected_without_replacing_canonical_identity() {
    let id = Uuid::from_u128(1);
    let canonical = Action {
        action_id: id,
        request: check(id),
        phase: Phase::Completed,
        error: None,
    };
    let shared = Arc::new(Mutex::new(State::ready(target(), Configuration::default())));
    let (sender, receiver) = mpsc::sync_channel(1);
    let runtime = Runtimes {
        shared: shared.clone(),
        sender: Some(sender),
        worker: None,
        stop: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    };
    let conflicting = save(id, Some("/other/codex"));
    runtime.request(conflicting.clone()).unwrap();
    let mut store = Memory::default();
    store.actions.insert(id, canonical.clone());
    execute(
        &shared,
        &mut store,
        receiver.try_recv().unwrap(),
        "boot",
        |_, _, _| panic!("conflicting payload executed"),
    );
    let status = runtime
        .request(RuntimeRequest::Action {
            target: target(),
            action_id: id,
        })
        .unwrap();
    assert_eq!(
        status.lookup.unwrap().action.unwrap().request,
        canonical.request
    );
    let rejection = status.rejected.unwrap();
    assert_eq!(rejection.request, conflicting);
    assert_eq!(rejection.error.code, ErrorCode::InvalidRequest);
    assert_eq!(
        runtime
            .request(canonical.request.clone())
            .unwrap()
            .lookup
            .unwrap()
            .action
            .unwrap()
            .request,
        canonical.request
    );
    assert_eq!(store.actions[&id].request, canonical.request);
}
