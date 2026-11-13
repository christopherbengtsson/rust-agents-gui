use super::*;

fn fixture() -> (Workspace, RepositoryRecord) {
    let repository = RepositoryRecord {
        id: RepositoryId(Uuid::new_v4()),
        name: "repo".into(),
        path: "/original".into(),
        common_dir: "/original/.git".into(),
    };
    let mut workspace = allocate(
        repository.id,
        ActionId(Uuid::new_v4()),
        Path::new("/private"),
    )
    .unwrap();
    workspace.path = Some("/private/workspace/checkout".into());
    workspace.branch = Some("workspace/fixture".into());
    workspace.name = Some("fixture".into());
    workspace.status = CreationStatus::Ready;
    (workspace, repository)
}
fn shared() -> Mutex<OwnerState> {
    Mutex::new(OwnerState {
        inspections: Inspections::default(),
        directory: "/unused".into(),
        store: Err(RepositoryError::storage("No real storage in units")),
        cache: VecDeque::new(),
        probes: VecDeque::new(),
        fault: None,
        revision: 1,
        active_workspace: None,
        workspace_faults: BTreeMap::new(),
    })
}
struct Effects<'a> {
    shared: &'a Mutex<OwnerState>,
    status: Vec<u8>,
    contents: Vec<u8>,
    patch: Vec<u8>,
    fail: bool,
    reads: usize,
    file: FileEvidence,
    heads: VecDeque<Result<String>>,
    statuses: VecDeque<Result<Vec<u8>>>,
    evidence: VecDeque<Result<FileEvidence>>,
    read_failure: Option<RepositoryError>,
    diff_failure: Option<RepositoryError>,
    diffs: usize,
}
impl<'a> Effects<'a> {
    fn new(shared: &'a Mutex<OwnerState>) -> Self {
        Self {
            shared,
            status: b"1 .M N... 100644 100644 100644 a b text.txt\0".to_vec(),
            contents: b"new\n".to_vec(),
            patch: b"@@ -1 +1 @@\n-old\n+new\n".to_vec(),
            fail: false,
            reads: 0,
            heads: VecDeque::new(),
            statuses: VecDeque::new(),
            evidence: VecDeque::new(),
            read_failure: None,
            diff_failure: None,
            diffs: 0,
            file: FileEvidence::Regular {
                device: 1,
                inode: 2,
                size: 4,
                modified: (3, 0),
                changed: (3, 0),
            },
        }
    }
}
impl InspectionEffects for Effects<'_> {
    fn validate(&mut self, workspace: &Workspace, repository: &RepositoryRecord) -> Result<()> {
        assert_eq!(workspace.repository_id, repository.id);
        assert_eq!(workspace.path()?, "/private/workspace/checkout");
        let state = self
            .shared
            .try_lock()
            .expect("effects must not hold the observation/storage lock");
        assert!(state.inspections.active.is_some());
        Ok(())
    }
    fn head(&mut self, _: &Path) -> Result<String> {
        if self.fail {
            Err(invalid("Injected Git failure"))
        } else {
            self.heads.pop_front().unwrap_or_else(|| Ok("a".repeat(40)))
        }
    }
    fn status(&mut self, _: &Path) -> Result<Vec<u8>> {
        self.statuses
            .pop_front()
            .unwrap_or_else(|| Ok(self.status.clone()))
    }
    fn evidence(&mut self, _: &Path, _: &[u8]) -> Result<FileEvidence> {
        self.evidence
            .pop_front()
            .unwrap_or_else(|| Ok(self.file.clone()))
    }
    fn read(&mut self, _: &Path, _: &[u8]) -> Result<Vec<u8>> {
        self.reads += 1;
        match &self.read_failure {
            Some(error) => Err(error.clone()),
            None => Ok(self.contents.clone()),
        }
    }
    fn diff(&mut self, _: &Path, _: &str, _: &[u8]) -> Result<Vec<u8>> {
        self.diffs += 1;
        match &self.diff_failure {
            Some(error) => Err(error.clone()),
            None => Ok(self.patch.clone()),
        }
    }
}
fn observation(reply: WorkspaceReply) -> ChangesObservation {
    match reply {
        WorkspaceReply::Changes { observation } => observation,
        _ => panic!("wrong reply"),
    }
}
fn refreshed(
    shared: &Mutex<OwnerState>,
    workspace: &Workspace,
    repository: &RepositoryRecord,
    effects: &mut Effects<'_>,
) -> ChangePage {
    let reply = shared
        .lock()
        .unwrap()
        .inspections
        .changes(workspace.clone(), repository.clone(), true, None, None)
        .unwrap();
    assert_eq!(observation(reply).phase, InspectionPhase::Queued);
    assert!(process_inspection(shared, effects));
    observation(
        shared
            .lock()
            .unwrap()
            .inspections
            .changes(workspace.clone(), repository.clone(), false, None, None)
            .unwrap(),
    )
    .page
    .unwrap()
}

#[test]
fn manual_inspection_coalesces_and_retains_last_good_after_git_failure() {
    let shared = shared();
    let (workspace, repository) = fixture();
    let mut effects = Effects::new(&shared);
    for _ in 0..5 {
        shared
            .lock()
            .unwrap()
            .inspections
            .changes(workspace.clone(), repository.clone(), true, None, None)
            .unwrap();
    }
    assert_eq!(shared.lock().unwrap().inspections.pending.len(), 1);
    assert!(process_inspection(&shared, &mut effects));
    let reply = observation(
        shared
            .lock()
            .unwrap()
            .inspections
            .changes(workspace.clone(), repository.clone(), false, None, None)
            .unwrap(),
    );
    let page = reply.page.unwrap();
    let target = PreviewTarget {
        workspace_id: workspace.id,
        comparison_id: page.comparison_id,
        file_id: page.files[0].id,
    };
    shared
        .lock()
        .unwrap()
        .inspections
        .preview(workspace.clone(), repository.clone(), target.clone(), true)
        .unwrap();
    assert!(process_inspection(&shared, &mut effects));
    let preview = shared
        .lock()
        .unwrap()
        .inspections
        .record(workspace.id)
        .unwrap()
        .preview
        .clone()
        .unwrap();
    assert_eq!(preview.preview.unwrap().rows[2].text, "new");
    effects.fail = true;
    shared
        .lock()
        .unwrap()
        .inspections
        .changes(workspace.clone(), repository.clone(), true, None, None)
        .unwrap();
    assert!(process_inspection(&shared, &mut effects));
    let failed = observation(
        shared
            .lock()
            .unwrap()
            .inspections
            .changes(workspace, repository, false, None, None)
            .unwrap(),
    );
    assert_eq!(failed.phase, InspectionPhase::Failed);
    assert!(failed.error.is_some());
    assert_eq!(failed.page.unwrap().comparison_id, page.comparison_id);
    assert!(!process_inspection(&shared, &mut effects));
}

#[test]
fn selected_file_failure_retains_previous_comparison_with_explicit_identity() {
    let shared = shared();
    let (workspace, repository) = fixture();
    let mut effects = Effects::new(&shared);
    let page = refreshed(&shared, &workspace, &repository, &mut effects);
    let mut target = PreviewTarget {
        workspace_id: workspace.id,
        comparison_id: page.comparison_id,
        file_id: page.files[0].id,
    };
    shared
        .lock()
        .unwrap()
        .inspections
        .preview(workspace.clone(), repository.clone(), target.clone(), true)
        .unwrap();
    process_inspection(&shared, &mut effects);
    let next = refreshed(&shared, &workspace, &repository, &mut effects);
    target.comparison_id = next.comparison_id;
    shared
        .lock()
        .unwrap()
        .inspections
        .preview(workspace.clone(), repository.clone(), target.clone(), true)
        .unwrap();
    effects.fail = true;
    process_inspection(&shared, &mut effects);
    let record = shared.lock().unwrap();
    let p = record.inspections.records[0].preview.as_ref().unwrap();
    assert_eq!(p.phase, InspectionPhase::Failed);
    assert_eq!(p.target, target);
    assert_eq!(
        p.preview.as_ref().unwrap().comparison_id,
        page.comparison_id
    );
}

#[test]
fn list_pages_expire_and_responses_stay_bounded() {
    let shared = shared();
    let (workspace, repository) = fixture();
    let mut effects = Effects::new(&shared);
    effects.status = (0..70)
        .flat_map(|n| format!("? file-{n}\0").into_bytes())
        .collect();
    let first = refreshed(&shared, &workspace, &repository, &mut effects);
    assert_eq!(first.files.len(), 32);
    assert_eq!(first.total, 70);
    let reply = shared
        .lock()
        .unwrap()
        .inspections
        .changes(
            workspace.clone(),
            repository.clone(),
            false,
            Some(first.comparison_id),
            first.next,
        )
        .unwrap();
    assert!(serde_json::to_vec(&reply).unwrap().len() <= RESPONSE_BYTES);
    assert_eq!(observation(reply).page.unwrap().files.len(), 32);
    refreshed(&shared, &workspace, &repository, &mut effects);
    assert!(
        shared
            .lock()
            .unwrap()
            .inspections
            .changes(
                workspace,
                repository,
                false,
                Some(first.comparison_id),
                first.next
            )
            .is_err()
    );
}

#[test]
fn previews_bound_rows_and_escaped_bytes_without_diff_pagination() {
    let target = PreviewTarget {
        workspace_id: WorkspaceId(Uuid::new_v4()),
        comparison_id: ComparisonId(Uuid::new_v4()),
        file_id: FileId(Uuid::new_v4()),
    };
    let make = |text: String| FilePreview {
        comparison_id: target.comparison_id,
        head: "a".repeat(40),
        rows: (1..=300)
            .map(|n| DiffRow {
                kind: RowKind::Added,
                old_line: None,
                new_line: Some(n),
                text: text.clone(),
            })
            .collect(),
        limited: None,
        unsupported: None,
        note: None,
    };
    let bounded = bound_preview(&target, make("small".into())).unwrap();
    assert_eq!(bounded.preview.as_ref().unwrap().rows.len(), 200);
    assert!(bounded.preview.as_ref().unwrap().limited.is_some());
    let bounded = bound_preview(&target, make("\"\\\t".repeat(1000))).unwrap();
    assert!(
        serde_json::to_vec(&WorkspaceReply::ChangePreview {
            observation: bounded.clone()
        })
        .unwrap()
        .len()
            <= RESPONSE_BYTES
    );
    assert!(bounded.preview.unwrap().rows.len() < 200);
    let bounded = bound_preview(&target, make("x".repeat(RESPONSE_BYTES))).unwrap();
    assert_eq!(
        bounded.preview.unwrap().unsupported,
        Some(UnsupportedContent::PreviewLimit)
    );
}

#[test]
fn unsupported_content_is_not_read_and_failed_diff_is_not_a_prefix() {
    let shared = shared();
    let (workspace, repository) = fixture();
    let mut effects = Effects::new(&shared);
    effects.file = FileEvidence::Unsupported(UnsupportedContent::SpecialFile);
    let page = refreshed(&shared, &workspace, &repository, &mut effects);
    assert_eq!(
        page.files[0].unsupported,
        Some(UnsupportedContent::SpecialFile)
    );
    let target = PreviewTarget {
        workspace_id: workspace.id,
        comparison_id: page.comparison_id,
        file_id: page.files[0].id,
    };
    shared
        .lock()
        .unwrap()
        .inspections
        .preview(workspace, repository, target, true)
        .unwrap();
    process_inspection(&shared, &mut effects);
    assert_eq!(effects.reads, 0);
    assert!(parse_diff(b"@@ -1,2 +1,2 @@\n-old\n+new\n").is_err());
}

#[test]
fn existing_jobs_and_inspection_take_fair_turns() {
    let mut next = 0;
    let mut order = Vec::new();
    for _ in 0..9 {
        assert!(worker_cycle(&mut next, |kind| {
            order.push(kind);
            true
        }));
    }
    assert_eq!(order, vec![0, 1, 2, 0, 1, 2, 0, 1, 2]);
    assert!(worker_cycle(&mut next, |kind| kind == 2));
    assert_eq!(next, 0);
}

#[test]
fn manual_text_add_delete_and_clean_outcomes_are_distinct() {
    for (status, content, patch, expected_kind, expected_note) in [
        (
            b"? new.txt\0".as_slice(),
            b"new\n".as_slice(),
            b"".as_slice(),
            Some(RowKind::Added),
            None,
        ),
        (b"? empty.txt\0", b"", b"", None, Some("Empty new file.")),
        (
            b"1 .D N... 100644 100644 000000 a b deleted.txt\0",
            b"",
            b"@@ -1 +0,0 @@\n-old\n",
            Some(RowKind::Removed),
            None,
        ),
        (
            b"1 .D N... 100644 100644 000000 a b empty.txt\0",
            b"",
            b"diff --git a/empty.txt b/empty.txt\ndeleted file mode 100644\nindex a..b\n",
            None,
            Some("Empty deleted file."),
        ),
    ] {
        let shared = shared();
        let (workspace, repository) = fixture();
        let mut effects = Effects::new(&shared);
        effects.status = status.to_vec();
        effects.contents = content.to_vec();
        effects.patch = patch.to_vec();
        if status.starts_with(b"1 .D") {
            effects.file = FileEvidence::Missing;
        }
        let page = refreshed(&shared, &workspace, &repository, &mut effects);
        let target = PreviewTarget {
            workspace_id: workspace.id,
            comparison_id: page.comparison_id,
            file_id: page.files[0].id,
        };
        shared
            .lock()
            .unwrap()
            .inspections
            .preview(workspace.clone(), repository.clone(), target, true)
            .unwrap();
        process_inspection(&shared, &mut effects);
        let p = shared.lock().unwrap().inspections.records[0]
            .preview
            .clone()
            .unwrap()
            .preview
            .unwrap();
        assert_eq!(
            p.rows
                .iter()
                .find(|r| r.kind != RowKind::Hunk)
                .map(|r| r.kind),
            expected_kind
        );
        assert_eq!(p.note.as_deref(), expected_note);
        if status.starts_with(b"1 .D") {
            assert_eq!(effects.reads, 0);
        }
    }
    let shared = shared();
    let (workspace, repository) = fixture();
    let mut effects = Effects::new(&shared);
    effects.status.clear();
    let page = refreshed(&shared, &workspace, &repository, &mut effects);
    assert_eq!(page.total, 0);
    assert!(page.files.is_empty());
    assert!(page.next.is_none());
    assert!(parse_status(b"\0").is_err());
}

#[test]
fn unsafe_content_labels_and_changed_evidence_do_not_read_text() {
    for (status, reason) in [
        (
            b"1 .T N... 100644 100644 120000 a b link\0".as_slice(),
            UnsupportedContent::Symlink,
        ),
        (
            b"1 .M S.M. 160000 160000 160000 a b module\0",
            UnsupportedContent::Submodule,
        ),
        (
            b"u UU N... 100644 100644 100644 100644 a b c conflict\0",
            UnsupportedContent::Conflict,
        ),
        (b"? non-utf8-\xff\0", UnsupportedContent::NonUtf8),
    ] {
        assert_eq!(parse_status(status).unwrap()[0].unsupported, Some(reason));
    }
    let shared = shared();
    let (workspace, repository) = fixture();
    let mut effects = Effects::new(&shared);
    let page = refreshed(&shared, &workspace, &repository, &mut effects);
    let target = PreviewTarget {
        workspace_id: workspace.id,
        comparison_id: page.comparison_id,
        file_id: page.files[0].id,
    };
    shared
        .lock()
        .unwrap()
        .inspections
        .preview(workspace, repository, target, true)
        .unwrap();
    effects.file = FileEvidence::Missing;
    process_inspection(&shared, &mut effects);
    assert_eq!(effects.reads, 0);
    assert_eq!(
        shared.lock().unwrap().inspections.records[0]
            .preview
            .as_ref()
            .unwrap()
            .phase,
        InspectionPhase::Failed
    );
}

#[test]
fn list_byte_budget_and_pending_retention_are_bounded() {
    let shared = shared();
    let (workspace, repository) = fixture();
    let mut effects = Effects::new(&shared);
    effects.status = (0..80)
        .flat_map(|n| format!("? {n}-{}\0", "\t".repeat(1000)).into_bytes())
        .collect();
    let page = refreshed(&shared, &workspace, &repository, &mut effects);
    assert!(page.files.len() < 32);
    assert!(page.next.is_some());
    assert_eq!(page.total, 80);
    let mut owner = Inspections::default();
    for _ in 0..8 {
        let mut w = workspace.clone();
        w.id = WorkspaceId(Uuid::new_v4());
        owner
            .changes(w, repository.clone(), true, None, None)
            .unwrap();
    }
    assert!(
        owner
            .changes(workspace, repository, true, None, None)
            .is_err()
    );
    assert_eq!(owner.pending.len(), 8);
    assert_eq!(owner.records.len(), 8);
}

#[test]
fn a_duplicate_active_preview_can_be_selected_again_without_losing_completion() {
    let shared = shared();
    let (workspace, repository) = fixture();
    let mut effects = Effects::new(&shared);
    let page = refreshed(&shared, &workspace, &repository, &mut effects);
    let target = PreviewTarget {
        workspace_id: workspace.id,
        comparison_id: page.comparison_id,
        file_id: page.files[0].id,
    };
    let mut state = shared.lock().unwrap();
    state
        .inspections
        .preview(workspace.clone(), repository.clone(), target.clone(), true)
        .unwrap();
    state.inspections.active = state.inspections.pending.pop_front();
    state.inspections.records[0].preview = None; // another selection superseded presentation
    let reply = state
        .inspections
        .preview(workspace, repository, target, true)
        .unwrap();
    let WorkspaceReply::ChangePreview { observation } = reply else {
        panic!()
    };
    assert_eq!(observation.phase, InspectionPhase::Running);
    assert!(state.inspections.pending.is_empty());
}

#[test]
fn escaped_failure_details_rebudget_the_last_good_preview() {
    let target = PreviewTarget {
        workspace_id: WorkspaceId(Uuid::new_v4()),
        comparison_id: ComparisonId(Uuid::new_v4()),
        file_id: FileId(Uuid::new_v4()),
    };
    let preview = FilePreview {
        comparison_id: target.comparison_id,
        head: "a".repeat(40),
        rows: (1..=200)
            .map(|n| DiffRow {
                kind: RowKind::Added,
                old_line: None,
                new_line: Some(n),
                text: "x".repeat(400),
            })
            .collect(),
        limited: None,
        unsupported: None,
        note: None,
    };
    let mut p = bound_preview(&target, preview).unwrap();
    p.phase = InspectionPhase::Failed;
    p.error = Some(RepositoryError::new(ErrorCode::Git, "\u{1}".repeat(2048)));
    bound_failed_preview(&mut p);
    assert!(
        serde_json::to_vec(&WorkspaceReply::ChangePreview { observation: p })
            .unwrap()
            .len()
            <= RESPONSE_BYTES
    );
}

#[test]
fn binary_marker_words_inside_text_hunks_are_regular_content() {
    let shared = shared();
    let (workspace, repository) = fixture();
    let mut effects = Effects::new(&shared);
    effects.contents = b"Binary files are discussed here\nGIT binary patch\n".to_vec();
    effects.patch =
        b"@@ -1 +1,2 @@\n-old\n+Binary files are discussed here\n+GIT binary patch\n".to_vec();
    let page = refreshed(&shared, &workspace, &repository, &mut effects);
    let target = PreviewTarget {
        workspace_id: workspace.id,
        comparison_id: page.comparison_id,
        file_id: page.files[0].id,
    };
    shared
        .lock()
        .unwrap()
        .inspections
        .preview(workspace, repository, target, true)
        .unwrap();
    process_inspection(&shared, &mut effects);
    let p = shared.lock().unwrap().inspections.records[0]
        .preview
        .clone()
        .unwrap()
        .preview
        .unwrap();
    assert_eq!(p.unsupported, None);
    assert_eq!(p.rows.len(), 4);
}

#[test]
fn a_hunk_header_does_not_hide_an_individually_oversized_content_row() {
    let target = PreviewTarget {
        workspace_id: WorkspaceId(Uuid::new_v4()),
        comparison_id: ComparisonId(Uuid::new_v4()),
        file_id: FileId(Uuid::new_v4()),
    };
    let p = FilePreview {
        comparison_id: target.comparison_id,
        head: "a".repeat(40),
        rows: vec![
            DiffRow {
                kind: RowKind::Hunk,
                old_line: None,
                new_line: None,
                text: "@@ -0,0 +1 @@".into(),
            },
            DiffRow {
                kind: RowKind::Added,
                old_line: None,
                new_line: Some(1),
                text: "x".repeat(50 * 1024),
            },
        ],
        limited: None,
        unsupported: None,
        note: None,
    };
    let p = bound_preview(&target, p).unwrap();
    assert_eq!(
        p.preview.as_ref().unwrap().unsupported,
        Some(UnsupportedContent::PreviewLimit)
    );
    assert!(
        serde_json::to_vec(&WorkspaceReply::ChangePreview { observation: p })
            .unwrap()
            .len()
            <= RESPONSE_BYTES
    );
}

#[test]
fn status_keeps_index_worktree_and_raw_path_identity() {
    let files =
        parse_status(b"1 MM N... 100644 100644 100644 a b src/a b.txt\0? new.txt\0").unwrap();
    assert_eq!(files.len(), 2);
    assert_eq!(files[0].path, b"src/a b.txt");
    assert_eq!(files[0].index_status, "M");
    assert_eq!(files[0].worktree_status, "M");
    assert_eq!(files[1].index_status, "?");
    assert!(parse_status(b"? ../escape\0").is_err());
    assert!(parse_status(b"? incomplete").is_err());
}

#[test]
fn unified_rows_keep_source_numbers_and_missing_newline_markers() {
    let rows = parse_diff(b"diff --git a/a b/a\n--- a/a\n+++ b/a\n@@ -7,2 +7,2 @@ name\n same\n-old\n+new\n\\ No newline at end of file\n").unwrap();
    assert_eq!(rows.len(), 5);
    assert_eq!((rows[1].old_line, rows[1].new_line), (Some(7), Some(7)));
    assert_eq!((rows[2].old_line, rows[2].new_line), (Some(8), None));
    assert_eq!((rows[3].old_line, rows[3].new_line), (None, Some(8)));
    assert_eq!(rows[4].kind, RowKind::NoNewline);
    assert!(parse_diff(b"@@ -7,2 +7,2 @@\n-short\n").is_err());
}

#[test]
fn refreshed_comparisons_invalidate_queued_previews_before_effects() {
    let shared = shared();
    let (workspace, repository) = fixture();
    let mut effects = Effects::new(&shared);
    let page = refreshed(&shared, &workspace, &repository, &mut effects);
    let target = PreviewTarget {
        workspace_id: workspace.id,
        comparison_id: page.comparison_id,
        file_id: page.files[0].id,
    };
    {
        let mut state = shared.lock().unwrap();
        state
            .inspections
            .changes(workspace.clone(), repository.clone(), true, None, None)
            .unwrap();
        state
            .inspections
            .preview(workspace.clone(), repository.clone(), target.clone(), true)
            .unwrap();
    }
    assert!(process_inspection(&shared, &mut effects)); // New comparison precedes old queued preview.
    assert!(!process_inspection(&shared, &mut effects));
    assert_eq!(effects.reads, 0);
    let mut state = shared.lock().unwrap();
    assert!(
        state
            .inspections
            .preview(workspace, repository, target, false)
            .is_err()
    );
    let retained = state.inspections.records[0].preview.as_ref().unwrap();
    assert_eq!(retained.phase, InspectionPhase::Failed);
    assert_eq!(retained.error.as_ref().unwrap().code, ErrorCode::Conflict);
}

fn load_preview(
    shared: &Mutex<OwnerState>,
    workspace: &Workspace,
    repository: &RepositoryRecord,
    target: &PreviewTarget,
    effects: &mut Effects<'_>,
) -> PreviewObservation {
    shared
        .lock()
        .unwrap()
        .inspections
        .preview(workspace.clone(), repository.clone(), target.clone(), true)
        .unwrap();
    assert!(process_inspection(shared, effects));
    let WorkspaceReply::ChangePreview { observation } = shared
        .lock()
        .unwrap()
        .inspections
        .preview(workspace.clone(), repository.clone(), target.clone(), false)
        .unwrap()
    else {
        panic!("preview observation expected")
    };
    observation
}

#[test]
fn refreshed_same_status_keeps_file_identity_and_replaces_selected_content() {
    let shared = shared();
    let (workspace, repository) = fixture();
    let mut effects = Effects::new(&shared);
    let first = refreshed(&shared, &workspace, &repository, &mut effects);
    let mut target = PreviewTarget {
        workspace_id: workspace.id,
        comparison_id: first.comparison_id,
        file_id: first.files[0].id,
    };
    let old = load_preview(&shared, &workspace, &repository, &target, &mut effects)
        .preview
        .unwrap();
    effects.contents = b"further edit\n".to_vec();
    effects.patch = b"@@ -1 +1 @@\n-old\n+further edit\n".to_vec();
    effects.file = FileEvidence::Regular {
        device: 1,
        inode: 2,
        size: 13,
        modified: (4, 0),
        changed: (4, 0),
    };
    let next = refreshed(&shared, &workspace, &repository, &mut effects);
    assert_ne!(first.comparison_id, next.comparison_id);
    assert_eq!(first.files[0].id, next.files[0].id);
    assert_eq!(
        first.files[0].worktree_status,
        next.files[0].worktree_status
    );
    assert!(
        shared
            .lock()
            .unwrap()
            .inspections
            .preview(workspace.clone(), repository.clone(), target.clone(), false)
            .is_err()
    );
    target.comparison_id = next.comparison_id;
    let new = load_preview(&shared, &workspace, &repository, &target, &mut effects)
        .preview
        .unwrap();
    assert_eq!(old.rows[2].text, "new");
    assert_eq!(new.rows[2].text, "further edit");
    assert_eq!(new.comparison_id, next.comparison_id);
}

#[test]
fn supplied_read_and_diff_failures_retain_last_good_then_clear_on_success() {
    for fail_read in [true, false] {
        let shared = shared();
        let (workspace, repository) = fixture();
        let mut effects = Effects::new(&shared);
        let page = refreshed(&shared, &workspace, &repository, &mut effects);
        let mut target = PreviewTarget {
            workspace_id: workspace.id,
            comparison_id: page.comparison_id,
            file_id: page.files[0].id,
        };
        let good = load_preview(&shared, &workspace, &repository, &target, &mut effects)
            .preview
            .unwrap();
        let next = refreshed(&shared, &workspace, &repository, &mut effects);
        target.comparison_id = next.comparison_id;
        if fail_read {
            effects.read_failure = Some(invalid("Supplied read failure"));
        } else {
            effects.diff_failure = Some(invalid("Supplied Git output-budget failure"));
        }
        let failure = load_preview(&shared, &workspace, &repository, &target, &mut effects);
        assert_eq!(failure.phase, InspectionPhase::Failed);
        let retained = failure.preview.unwrap();
        assert_eq!(retained.comparison_id, good.comparison_id);
        assert_eq!(retained.rows, good.rows);
        assert!(failure.error.is_some());
        effects.read_failure = None;
        effects.diff_failure = None;
        let recovered = load_preview(&shared, &workspace, &repository, &target, &mut effects);
        assert_eq!(recovered.phase, InspectionPhase::Ready);
        assert!(recovered.error.is_none());
        assert_eq!(recovered.preview.unwrap().comparison_id, next.comparison_id);
    }
}

#[test]
fn supplied_content_outcomes_keep_unsupported_empty_and_net_cancellation_distinct() {
    for (status, contents, patch, expected, note) in [
        (
            b"? added\0".as_slice(),
            b"\0binary".as_slice(),
            b"".as_slice(),
            Some(UnsupportedContent::Binary),
            None,
        ),
        (
            b"? added\0",
            b"bad \xff",
            b"",
            Some(UnsupportedContent::NonUtf8),
            None,
        ),
        (
            b"1 .M N... 100644 100644 100644 a b text.txt\0",
            b"new\n",
            b"Binary files a/text.txt and b/text.txt differ\n",
            Some(UnsupportedContent::Binary),
            None,
        ),
        (
            b"1 .M N... 100644 100644 100644 a b text.txt\0",
            b"new\n",
            b"@@ -1 +1 @@\n-old \xff\n+new\n",
            Some(UnsupportedContent::NonUtf8),
            None,
        ),
        (
            b"1 MM N... 100644 100644 100644 a b text.txt\0",
            b"new\n",
            b"",
            None,
            Some("No net text change against HEAD. Staged and unstaged changes may cancel."),
        ),
        (
            b"1 M. N... 100644 100755 100755 a b text.txt\0",
            b"new\n",
            b"diff --git a/text.txt b/text.txt\nold mode 100644\nnew mode 100755\n",
            None,
            Some("Metadata-only change."),
        ),
        (
            b"1 A. N... 000000 100644 100644 a b empty\0",
            b"",
            b"diff --git a/empty b/empty\nnew file mode 100644\n",
            None,
            Some("Empty added file."),
        ),
    ] {
        let shared = shared();
        let (workspace, repository) = fixture();
        let mut effects = Effects::new(&shared);
        effects.status = status.to_vec();
        effects.contents = contents.to_vec();
        effects.patch = patch.to_vec();
        let page = refreshed(&shared, &workspace, &repository, &mut effects);
        let target = PreviewTarget {
            workspace_id: workspace.id,
            comparison_id: page.comparison_id,
            file_id: page.files[0].id,
        };
        let result = load_preview(&shared, &workspace, &repository, &target, &mut effects);
        assert_eq!(result.phase, InspectionPhase::Ready);
        assert!(result.error.is_none());
        let preview = result.preview.unwrap();
        assert_eq!(preview.unsupported, expected);
        assert_eq!(preview.note.as_deref(), note);
        assert!(preview.rows.is_empty());
        assert_eq!(page.total, 1); // An empty comparison is not a clean inventory.
    }
}

#[test]
fn line_endings_and_untracked_missing_final_newline_preserve_bytes_and_numbers() {
    let rows = parse_diff(b"@@ -1 +1 @@\n-old\r\n+new\n").unwrap();
    assert_eq!(rows[1].text, "old\r");
    assert_eq!(rows[2].text, "new");
    assert_eq!((rows[1].old_line, rows[2].new_line), (Some(1), Some(1)));
    for content in [b"one\r\ntwo".as_slice(), b"one\ntwo", b"one\n", b"", b"\n"] {
        let shared = shared();
        let (workspace, repository) = fixture();
        let mut effects = Effects::new(&shared);
        effects.status = b"? added\0".to_vec();
        effects.contents = content.to_vec();
        let page = refreshed(&shared, &workspace, &repository, &mut effects);
        let target = PreviewTarget {
            workspace_id: workspace.id,
            comparison_id: page.comparison_id,
            file_id: page.files[0].id,
        };
        let p = load_preview(&shared, &workspace, &repository, &target, &mut effects)
            .preview
            .unwrap();
        let added: Vec<_> = p.rows.iter().filter(|r| r.kind == RowKind::Added).collect();
        match content {
            b"one\r\ntwo" => {
                assert_eq!(added[0].text, "one\r");
                assert_eq!(added[1].text, "two");
                assert_eq!(added[1].new_line, Some(2));
            }
            b"one\ntwo" => {
                assert_eq!(added[0].text, "one");
                assert_eq!(added[1].new_line, Some(2));
            }
            b"one\n" => {
                assert_eq!(added.len(), 1);
                assert_eq!(added[0].text, "one");
            }
            b"" => {
                assert!(added.is_empty());
                assert_eq!(p.note.as_deref(), Some("Empty new file."));
            }
            b"\n" => {
                assert_eq!(added.len(), 1);
                assert_eq!(added[0].text, "");
            }
            _ => unreachable!(),
        }
        assert_eq!(
            p.rows.iter().any(|r| r.kind == RowKind::NoNewline),
            content.ends_with(b"two")
        );
        assert_eq!(effects.diffs, 0);
    }
}

#[test]
fn unsafe_inventory_entries_never_read_or_diff_as_text() {
    for (status, evidence, expected) in [
        (
            b"1 .T N... 100644 100644 120000 a b link\0".as_slice(),
            None,
            UnsupportedContent::Symlink,
        ),
        (
            b"1 .M S.M. 160000 160000 160000 a b module\0",
            None,
            UnsupportedContent::Submodule,
        ),
        (
            b"u UU N... 100644 100644 100644 100644 a b c conflict\0",
            None,
            UnsupportedContent::Conflict,
        ),
        (b"? bad-\xff\0", None, UnsupportedContent::NonUtf8),
        (
            b"? special\0",
            Some(FileEvidence::Unsupported(UnsupportedContent::SpecialFile)),
            UnsupportedContent::SpecialFile,
        ),
        (
            b"? link\0",
            Some(FileEvidence::Unsupported(UnsupportedContent::Symlink)),
            UnsupportedContent::Symlink,
        ),
        (
            b"? huge\0",
            Some(FileEvidence::Unsupported(UnsupportedContent::Oversized)),
            UnsupportedContent::Oversized,
        ),
    ] {
        let shared = shared();
        let (workspace, repository) = fixture();
        let mut effects = Effects::new(&shared);
        effects.status = status.to_vec();
        if let Some(evidence) = evidence {
            effects.file = evidence;
        }
        let page = refreshed(&shared, &workspace, &repository, &mut effects);
        let target = PreviewTarget {
            workspace_id: workspace.id,
            comparison_id: page.comparison_id,
            file_id: page.files[0].id,
        };
        let result = load_preview(&shared, &workspace, &repository, &target, &mut effects);
        assert_eq!(result.phase, InspectionPhase::Ready);
        assert_eq!(result.preview.unwrap().unsupported, Some(expected));
        assert_eq!((effects.reads, effects.diffs), (0, 0));
    }
}

#[test]
fn head_status_and_selected_evidence_changes_reject_obsolete_preview_reads() {
    enum Changed {
        HeadBefore,
        HeadAfter,
        StatusBefore,
        StatusAfter,
        FileBefore,
        FileAfter,
    }
    for changed in [
        Changed::HeadBefore,
        Changed::HeadAfter,
        Changed::StatusBefore,
        Changed::StatusAfter,
        Changed::FileBefore,
        Changed::FileAfter,
    ] {
        let shared = shared();
        let (workspace, repository) = fixture();
        let mut effects = Effects::new(&shared);
        let page = refreshed(&shared, &workspace, &repository, &mut effects);
        let target = PreviewTarget {
            workspace_id: workspace.id,
            comparison_id: page.comparison_id,
            file_id: page.files[0].id,
        };
        let reads = match changed {
            Changed::HeadBefore => {
                effects.heads.push_back(Ok("b".repeat(40)));
                0
            }
            Changed::HeadAfter => {
                effects
                    .heads
                    .extend([Ok("a".repeat(40)), Ok("b".repeat(40))]);
                1
            }
            Changed::StatusBefore => {
                effects.statuses.push_back(Ok(Vec::new()));
                0
            }
            Changed::StatusAfter => {
                effects
                    .statuses
                    .extend([Ok(effects.status.clone()), Ok(Vec::new())]);
                1
            }
            Changed::FileBefore => {
                effects.evidence.push_back(Ok(FileEvidence::Missing));
                0
            }
            Changed::FileAfter => {
                effects.evidence.extend([
                    Ok(effects.file.clone()),
                    Ok(FileEvidence::Unsupported(UnsupportedContent::Symlink)),
                ]);
                1
            }
        };
        let result = load_preview(&shared, &workspace, &repository, &target, &mut effects);
        assert_eq!(result.phase, InspectionPhase::Failed);
        assert_eq!(result.error.unwrap().code, ErrorCode::Conflict);
        assert!(result.preview.is_none());
        assert_eq!(effects.reads, reads);
    }
}

#[test]
fn inventory_capture_failures_do_not_establish_clean_and_can_recover() {
    enum Failure {
        HeadChanged,
        StatusChanged,
        StatusRead,
        FileEvidence,
        TooMany,
        TooLarge,
        Incomplete,
    }
    for failure in [
        Failure::HeadChanged,
        Failure::StatusChanged,
        Failure::StatusRead,
        Failure::FileEvidence,
        Failure::TooMany,
        Failure::TooLarge,
        Failure::Incomplete,
    ] {
        let shared = shared();
        let (workspace, repository) = fixture();
        let mut effects = Effects::new(&shared);
        match failure {
            Failure::HeadChanged => effects
                .heads
                .extend([Ok("a".repeat(40)), Ok("b".repeat(40))]),
            Failure::StatusChanged => effects
                .statuses
                .extend([Ok(effects.status.clone()), Ok(Vec::new())]),
            Failure::StatusRead => effects
                .statuses
                .push_back(Err(invalid("Supplied status failure"))),
            Failure::FileEvidence => effects
                .evidence
                .push_back(Err(invalid("Supplied metadata failure"))),
            Failure::TooMany => {
                effects.status = (0..4097)
                    .flat_map(|n| format!("? file{n}\0").into_bytes())
                    .collect()
            }
            Failure::TooLarge => effects.status = vec![b'x'; CONTENT_BYTES + 1],
            Failure::Incomplete => effects.status = b"? incomplete".to_vec(),
        }
        shared
            .lock()
            .unwrap()
            .inspections
            .changes(workspace.clone(), repository.clone(), true, None, None)
            .unwrap();
        assert!(process_inspection(&shared, &mut effects));
        let failed = observation(
            shared
                .lock()
                .unwrap()
                .inspections
                .changes(workspace.clone(), repository.clone(), false, None, None)
                .unwrap(),
        );
        assert_eq!(failed.phase, InspectionPhase::Failed);
        assert!(failed.error.is_some());
        assert!(failed.page.is_none());
        effects.status.clear();
        effects.heads.clear();
        effects.statuses.clear();
        effects.evidence.clear();
        let recovered = refreshed(&shared, &workspace, &repository, &mut effects);
        assert_eq!(recovered.total, 0);
        assert!(
            shared.lock().unwrap().inspections.records[0]
                .error
                .is_none()
        );
    }
}

#[test]
fn unusual_paths_keep_raw_identity_even_when_display_labels_collide() {
    let shared = shared();
    let (workspace, repository) = fixture();
    let mut effects = Effects::new(&shared);
    effects.status = b"? :(glob)*\t\n.txt\0? bad-\xff\0? bad-\xfe\0".to_vec();
    let page = refreshed(&shared, &workspace, &repository, &mut effects);
    assert_eq!(page.files[0].path, ":(glob)*\\t\\n.txt");
    assert_eq!(page.files[1].path, page.files[2].path);
    assert_ne!(page.files[1].id, page.files[2].id);
    let state = shared.lock().unwrap();
    let raw = &state.inspections.records[0]
        .comparison
        .as_ref()
        .unwrap()
        .files;
    assert_eq!(raw[0].path, b":(glob)*\t\n.txt");
    assert_eq!(raw[1].path, b"bad-\xff");
    assert_eq!(raw[2].path, b"bad-\xfe");
}

#[test]
fn acquired_oversized_content_is_unsupported_and_valid_hunks_limit_complete_rows() {
    let shared = shared();
    let (workspace, repository) = fixture();
    let mut effects = Effects::new(&shared);
    let page = refreshed(&shared, &workspace, &repository, &mut effects);
    let target = PreviewTarget {
        workspace_id: workspace.id,
        comparison_id: page.comparison_id,
        file_id: page.files[0].id,
    };
    effects.contents = vec![b'x'; CONTENT_BYTES + 1];
    let p = load_preview(&shared, &workspace, &repository, &target, &mut effects)
        .preview
        .unwrap();
    assert_eq!(p.unsupported, Some(UnsupportedContent::Oversized));
    assert_eq!(effects.diffs, 0);
    effects.contents = b"new\n".to_vec();
    effects.patch = b"@@ -0,0 +1,300 @@\n".to_vec();
    for n in 1..=300 {
        effects.patch.extend(format!("+line {n}\n").bytes());
    }
    let next = refreshed(&shared, &workspace, &repository, &mut effects);
    let target = PreviewTarget {
        comparison_id: next.comparison_id,
        ..target
    };
    let result = load_preview(&shared, &workspace, &repository, &target, &mut effects);
    let p = result.preview.as_ref().unwrap();
    assert!(p.limited.is_some());
    assert_eq!(p.rows.len(), 200);
    assert_eq!(p.rows[199].text, "line 199");
    assert_eq!(p.rows[199].new_line, Some(199));
    assert!(
        serde_json::to_vec(&WorkspaceReply::ChangePreview {
            observation: result
        })
        .unwrap()
        .len()
            <= RESPONSE_BYTES
    );
}
