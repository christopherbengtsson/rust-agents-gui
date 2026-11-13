//! Ephemeral uncommitted comparisons. Policy stays here; effects are supplied.
use super::*;

identity!(ComparisonId);
identity!(FileId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnsupportedContent {
    Binary,
    NonUtf8,
    Symlink,
    Submodule,
    Conflict,
    SpecialFile,
    Oversized,
    PreviewLimit,
}

#[derive(Debug, Clone)]
struct FileEntry {
    id: FileId,
    path: Vec<u8>,
    index_status: String,
    worktree_status: String,
    unsupported: Option<UnsupportedContent>,
    evidence: Option<FileEvidence>,
}

fn invalid(message: &str) -> RepositoryError {
    RepositoryError::new(ErrorCode::Git, message)
}

fn valid_path(path: &[u8]) -> bool {
    !path.is_empty()
        && !path.contains(&0)
        && path
            .split(|b| *b == b'/')
            .all(|part| !part.is_empty() && part != b"." && part != b"..")
}

fn parse_status(bytes: &[u8]) -> Result<Vec<FileEntry>> {
    if bytes.len() > CONTENT_BYTES {
        return Err(invalid(
            "Git inventory exceeds the bounded acquisition limit.",
        ));
    }
    if !bytes.is_empty() && bytes.last() != Some(&0) {
        return Err(invalid(
            "Incomplete Git status output; refresh after resolving the Git failure.",
        ));
    }
    let mut files = Vec::new();
    if bytes.is_empty() {
        return Ok(files);
    }
    for record in bytes[..bytes.len() - 1].split(|b| *b == 0) {
        if files.len() == 4096 {
            return Err(invalid(
                "Git inventory exceeds the 4096-file inspection limit; the workspace is not reported clean.",
            ));
        }
        let (path, xy, unsupported) = match record.first() {
            Some(b'?') if record.starts_with(b"? ") => (&record[2..], &b"??"[..], None),
            Some(b'1') | Some(b'u') => {
                let conflict = record[0] == b'u';
                let fields: Vec<_> = record
                    .splitn(if conflict { 11 } else { 9 }, |b| *b == b' ')
                    .collect();
                if fields.len() != if conflict { 11 } else { 9 } || fields[1].len() != 2 {
                    return Err(invalid("Invalid Git status record."));
                }
                let modes = &fields[3..if conflict { 7 } else { 6 }];
                let unsupported = if conflict {
                    Some(UnsupportedContent::Conflict)
                } else if modes.contains(&&b"120000"[..]) {
                    Some(UnsupportedContent::Symlink)
                } else if modes.contains(&&b"160000"[..]) || fields[2].starts_with(b"S") {
                    Some(UnsupportedContent::Submodule)
                } else {
                    None
                };
                (*fields.last().unwrap(), fields[1], unsupported)
            }
            _ => {
                return Err(invalid(
                    "Unsupported Git status record; inventory was not accepted.",
                ));
            }
        };
        if !valid_path(path) || !xy.iter().all(|b| b".MADRCUT?!".contains(b)) {
            return Err(invalid("Invalid path or change status in Git inventory."));
        }
        if files.iter().any(|f: &FileEntry| f.path == path) {
            return Err(invalid("Duplicate path in Git inventory."));
        }
        files.push(FileEntry {
            id: FileId(Uuid::new_v4()),
            path: path.to_vec(),
            index_status: (xy[0] as char).to_string(),
            worktree_status: (xy[1] as char).to_string(),
            unsupported: unsupported.or_else(|| {
                std::str::from_utf8(path)
                    .err()
                    .map(|_| UnsupportedContent::NonUtf8)
            }),
            evidence: None,
        });
    }
    Ok(files)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RowKind {
    Hunk,
    Context,
    Added,
    Removed,
    NoNewline,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffRow {
    pub kind: RowKind,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
    pub text: String,
}
fn range(value: &str, prefix: char) -> Result<(u32, u32)> {
    let value = value
        .strip_prefix(prefix)
        .ok_or_else(|| invalid("Invalid diff range."))?;
    let (start, count) = value.split_once(',').unwrap_or((value, "1"));
    Ok((
        start
            .parse()
            .map_err(|_| invalid("Invalid diff line number."))?,
        count
            .parse()
            .map_err(|_| invalid("Invalid diff line count."))?,
    ))
}
fn parse_diff(bytes: &[u8]) -> Result<Vec<DiffRow>> {
    let text = std::str::from_utf8(bytes).map_err(|_| invalid("Non-UTF-8 diff."))?;
    if !text.is_empty() && !text.ends_with('\n') {
        return Err(invalid("Incomplete diff output."));
    }
    let mut rows = Vec::new();
    let (mut old, mut new, mut old_left, mut new_left) = (0u32, 0u32, 0u32, 0u32);
    let mut hunk = false;
    for line in text.split_terminator('\n') {
        if line.starts_with("@@ ") {
            if old_left != 0 || new_left != 0 {
                return Err(invalid("Incomplete diff hunk."));
            }
            let mut fields = line.splitn(5, ' ');
            fields.next();
            (old, old_left) = range(fields.next().unwrap_or(""), '-')?;
            (new, new_left) = range(fields.next().unwrap_or(""), '+')?;
            if fields.next() != Some("@@") {
                return Err(invalid("Invalid diff hunk."));
            }
            hunk = true;
            rows.push(DiffRow {
                kind: RowKind::Hunk,
                old_line: None,
                new_line: None,
                text: line.into(),
            });
            continue;
        }
        if !hunk {
            if [
                "diff --git ",
                "index ",
                "--- ",
                "+++ ",
                "new file mode ",
                "deleted file mode ",
                "old mode ",
                "new mode ",
            ]
            .iter()
            .any(|prefix| line.starts_with(prefix))
            {
                continue;
            }
            return Err(invalid("Unrecognized diff metadata."));
        }
        let (kind, old_line, new_line) = if line == "\\ No newline at end of file" {
            (RowKind::NoNewline, None, None)
        } else {
            let (kind, uses_old, uses_new) = match line.as_bytes().first() {
                Some(b' ') => (RowKind::Context, true, true),
                Some(b'-') => (RowKind::Removed, true, false),
                Some(b'+') => (RowKind::Added, false, true),
                _ => return Err(invalid("Invalid diff row.")),
            };
            let numbers = (uses_old.then_some(old), uses_new.then_some(new));
            if uses_old {
                old_left = old_left
                    .checked_sub(1)
                    .ok_or_else(|| invalid("Excess diff rows."))?;
                old = old
                    .checked_add(1)
                    .ok_or_else(|| invalid("Diff line number overflow."))?;
            }
            if uses_new {
                new_left = new_left
                    .checked_sub(1)
                    .ok_or_else(|| invalid("Excess diff rows."))?;
                new = new
                    .checked_add(1)
                    .ok_or_else(|| invalid("Diff line number overflow."))?;
            }
            (kind, numbers.0, numbers.1)
        };
        rows.push(DiffRow {
            kind,
            old_line,
            new_line,
            text: if kind == RowKind::NoNewline {
                line
            } else {
                &line[1..]
            }
            .into(),
        });
    }
    if old_left != 0 || new_left != 0 {
        return Err(invalid("Incomplete diff hunk."));
    }
    Ok(rows)
}

const RESPONSE_BYTES: usize = 48 * 1024;
const RETAINED_WORKSPACES: usize = 8;
const PENDING_JOBS: usize = 8;
pub(crate) const CONTENT_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectionPhase {
    Idle,
    Queued,
    Running,
    Ready,
    Failed,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewTarget {
    pub workspace_id: WorkspaceId,
    pub comparison_id: ComparisonId,
    pub file_id: FileId,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedFile {
    pub id: FileId,
    pub path: String,
    pub index_status: String,
    pub worktree_status: String,
    pub unsupported: Option<UnsupportedContent>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePage {
    pub comparison_id: ComparisonId,
    pub head: String,
    pub files: Vec<ChangedFile>,
    pub total: usize,
    pub next: Option<FileId>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesObservation {
    pub workspace_id: WorkspaceId,
    pub phase: InspectionPhase,
    pub page: Option<ChangePage>,
    pub error: Option<RepositoryError>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePreview {
    pub comparison_id: ComparisonId,
    pub head: String,
    pub rows: Vec<DiffRow>,
    pub limited: Option<String>,
    pub unsupported: Option<UnsupportedContent>,
    pub note: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewObservation {
    pub target: PreviewTarget,
    pub phase: InspectionPhase,
    pub preview: Option<FilePreview>,
    pub error: Option<RepositoryError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FileEvidence {
    Missing,
    Regular {
        device: u64,
        inode: u64,
        size: u64,
        modified: (i64, i64),
        changed: (i64, i64),
    },
    Unsupported(UnsupportedContent),
}
pub(crate) trait InspectionEffects {
    fn validate(&mut self, workspace: &Workspace, repository: &RepositoryRecord) -> Result<()>;
    fn head(&mut self, path: &Path) -> Result<String>;
    fn status(&mut self, path: &Path) -> Result<Vec<u8>>;
    fn evidence(&mut self, root: &Path, path: &[u8]) -> Result<FileEvidence>;
    fn read(&mut self, root: &Path, path: &[u8]) -> Result<Vec<u8>>;
    fn diff(&mut self, root: &Path, head: &str, path: &[u8]) -> Result<Vec<u8>>;
}
#[derive(Clone)]
struct Comparison {
    id: ComparisonId,
    head: String,
    status: Vec<u8>,
    files: Vec<FileEntry>,
}
struct Inspection {
    workspace_id: WorkspaceId,
    phase: InspectionPhase,
    comparison: Option<Comparison>,
    error: Option<RepositoryError>,
    preview: Option<PreviewObservation>,
}
#[derive(Clone)]
enum Job {
    Refresh {
        workspace: Workspace,
        repository: RepositoryRecord,
    },
    Preview {
        workspace: Workspace,
        repository: RepositoryRecord,
        target: PreviewTarget,
        comparison: Arc<Comparison>,
        file: Arc<FileEntry>,
    },
}
impl Job {
    fn workspace_id(&self) -> WorkspaceId {
        match self {
            Self::Refresh { workspace, .. } | Self::Preview { workspace, .. } => workspace.id,
        }
    }
    fn same_work(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Refresh { workspace: a, .. }, Self::Refresh { workspace: b, .. }) => {
                a.id == b.id
            }
            (Self::Preview { target: a, .. }, Self::Preview { target: b, .. }) => a == b,
            _ => false,
        }
    }
}
#[derive(Default)]
pub(super) struct Inspections {
    records: VecDeque<Inspection>,
    pending: VecDeque<Job>,
    active: Option<Job>,
}
fn expired() -> RepositoryError {
    RepositoryError::new(
        ErrorCode::Conflict,
        "This comparison or file is no longer available. Refresh the file list; no different file was selected.",
    )
}
impl Inspections {
    fn record(&mut self, id: WorkspaceId) -> Result<&mut Inspection> {
        if !self.records.iter().any(|r| r.workspace_id == id) {
            if self.records.len() >= RETAINED_WORKSPACES {
                let evict = self.records.iter().position(|r| {
                    !self
                        .pending
                        .iter()
                        .any(|j| j.workspace_id() == r.workspace_id)
                        && self
                            .active
                            .as_ref()
                            .is_none_or(|j| j.workspace_id() != r.workspace_id)
                });
                if let Some(index) = evict {
                    self.records.remove(index);
                } else {
                    return Err(RepositoryError::new(
                        ErrorCode::Capacity,
                        "Inspection is busy; wait for pending work before retrying.",
                    ));
                }
            }
            self.records.push_back(Inspection {
                workspace_id: id,
                phase: InspectionPhase::Idle,
                comparison: None,
                error: None,
                preview: None,
            });
        }
        Ok(self
            .records
            .iter_mut()
            .find(|r| r.workspace_id == id)
            .unwrap())
    }
    fn queue(&mut self, job: Job) -> Result<bool> {
        if self
            .pending
            .iter()
            .chain(self.active.iter())
            .any(|j| j.same_work(&job))
        {
            return Ok(false);
        }
        // One pending preview per workspace; another selection supersedes it.
        if matches!(job, Job::Preview { .. }) {
            self.pending.retain(|j| {
                !(matches!(j, Job::Preview { .. }) && j.workspace_id() == job.workspace_id())
            });
        }
        if self.pending.len() >= PENDING_JOBS {
            return Err(RepositoryError::new(
                ErrorCode::Capacity,
                "Eight inspection jobs are pending; wait and refresh observations.",
            ));
        }
        self.pending.push_back(job);
        Ok(true)
    }
    fn changes(
        &mut self,
        workspace: Workspace,
        repository: RepositoryRecord,
        refresh: bool,
        comparison_id: Option<ComparisonId>,
        after: Option<FileId>,
    ) -> Result<WorkspaceReply> {
        if refresh && (after.is_some() || comparison_id.is_some()) {
            return Err(expired());
        }
        self.record(workspace.id)?;
        if refresh
            && self.queue(Job::Refresh {
                workspace: workspace.clone(),
                repository,
            })?
        {
            self.record(workspace.id)?.phase = InspectionPhase::Queued;
        }
        let record = self.record(workspace.id)?;
        let mut observation = ChangesObservation {
            workspace_id: workspace.id,
            phase: record.phase,
            page: None,
            error: record.error.clone(),
        };
        if let Some(comparison) = &record.comparison {
            if comparison_id.is_some_and(|id| id != comparison.id) {
                return Err(expired());
            }
            let start = match after {
                None => 0,
                Some(id) if comparison_id.is_some() => comparison
                    .files
                    .iter()
                    .position(|f| f.id == id)
                    .ok_or_else(expired)?
                    .checked_add(1)
                    .ok_or_else(expired)?,
                Some(_) => return Err(expired()),
            };
            observation.page = Some(ChangePage {
                comparison_id: comparison.id,
                head: comparison.head.clone(),
                files: vec![],
                total: comparison.files.len(),
                next: None,
            });
            for file in comparison.files.iter().skip(start).take(32) {
                let page = observation.page.as_mut().unwrap();
                page.files.push(ChangedFile {
                    id: file.id,
                    path: display_path(&file.path),
                    index_status: file.index_status.clone(),
                    worktree_status: file.worktree_status.clone(),
                    unsupported: file.unsupported,
                });
                page.next = (start + page.files.len() < page.total).then_some(file.id);
                let reply = WorkspaceReply::Changes {
                    observation: observation.clone(),
                };
                if serde_json::to_vec(&reply)
                    .map_err(RepositoryError::storage)?
                    .len()
                    > RESPONSE_BYTES
                {
                    let page = observation.page.as_mut().unwrap();
                    page.files.pop();
                    page.next = page.files.last().map(|f| f.id);
                    if page.files.is_empty() {
                        return Err(RepositoryError::new(
                            ErrorCode::Capacity,
                            "A changed path exceeds the file-list response budget; inventory is not clean.",
                        ));
                    }
                    break;
                }
            }
        } else if comparison_id.is_some() || after.is_some() {
            return Err(expired());
        }
        Ok(WorkspaceReply::Changes { observation })
    }
    fn preview(
        &mut self,
        workspace: Workspace,
        repository: RepositoryRecord,
        target: PreviewTarget,
        load: bool,
    ) -> Result<WorkspaceReply> {
        let record = self.record(workspace.id)?;
        let comparison = record
            .comparison
            .as_ref()
            .filter(|c| c.id == target.comparison_id)
            .ok_or_else(expired)?
            .clone();
        let file = comparison
            .files
            .iter()
            .find(|f| f.id == target.file_id)
            .ok_or_else(|| RepositoryError::new(ErrorCode::NotFound, "The selected file no longer appears in this comparison. Select another path explicitly."))?
            .clone();
        let existing = record
            .preview
            .as_ref()
            .filter(|p| p.target == target)
            .cloned();
        if load
            && existing.as_ref().is_none_or(|p| {
                p.phase == InspectionPhase::Failed || p.phase == InspectionPhase::Idle
            })
        {
            let job = Job::Preview {
                workspace: workspace.clone(),
                repository,
                target: target.clone(),
                comparison: Arc::new(comparison),
                file: Arc::new(file),
            };
            self.queue(job.clone())?;
            let phase = if self
                .active
                .as_ref()
                .is_some_and(|active| active.same_work(&job))
            {
                InspectionPhase::Running
            } else {
                InspectionPhase::Queued
            };
            let last_good = self
                .record(workspace.id)?
                .preview
                .as_ref()
                .filter(|p| p.target.file_id == target.file_id)
                .and_then(|p| p.preview.clone());
            self.record(workspace.id)?.preview = Some(PreviewObservation {
                target: target.clone(),
                phase,
                preview: last_good,
                error: existing.and_then(|p| p.error),
            });
        }
        let observation = self
            .record(workspace.id)?
            .preview
            .as_ref()
            .filter(|p| p.target == target)
            .cloned()
            .unwrap_or(PreviewObservation {
                target,
                phase: InspectionPhase::Idle,
                preview: None,
                error: None,
            });
        Ok(WorkspaceReply::ChangePreview { observation })
    }
}
fn display_path(path: &[u8]) -> String {
    String::from_utf8_lossy(path)
        .chars()
        .flat_map(|c| {
            if c.is_control() {
                c.escape_default().collect::<Vec<_>>()
            } else {
                vec![c]
            }
        })
        .collect()
}

fn capture(
    workspace: &Workspace,
    repository: &RepositoryRecord,
    effects: &mut impl InspectionEffects,
) -> Result<Comparison> {
    effects.validate(workspace, repository)?;
    let root = Path::new(workspace.path()?);
    let head = effects.head(root)?;
    let status = effects.status(root)?;
    let mut files = parse_status(&status)?;
    for file in &mut files {
        if file.unsupported.is_none() {
            let evidence = effects.evidence(root, &file.path)?;
            if let FileEvidence::Unsupported(reason) = evidence {
                file.unsupported = Some(reason);
            }
            file.evidence = Some(evidence);
        }
    }
    if effects.head(root)? != head || effects.status(root)? != status {
        return Err(expired());
    }
    Ok(Comparison {
        id: ComparisonId(Uuid::new_v4()),
        head,
        status,
        files,
    })
}
fn acquire_preview(
    workspace: &Workspace,
    repository: &RepositoryRecord,
    target: &PreviewTarget,
    comparison: &Comparison,
    file: &FileEntry,
    effects: &mut impl InspectionEffects,
) -> Result<PreviewObservation> {
    effects.validate(workspace, repository)?;
    let root = Path::new(workspace.path()?);
    if effects.head(root)? != comparison.head || effects.status(root)? != comparison.status {
        return Err(expired());
    }
    let mut preview = FilePreview {
        comparison_id: comparison.id,
        head: comparison.head.clone(),
        rows: vec![],
        limited: None,
        unsupported: file.unsupported,
        note: None,
    };
    if preview.unsupported.is_none() {
        let before = effects.evidence(root, &file.path)?;
        if file.evidence.as_ref() != Some(&before) {
            return Err(expired());
        }
        let bytes = if before == FileEvidence::Missing {
            None
        } else {
            Some(effects.read(root, &file.path)?)
        };
        if bytes.as_ref().is_some_and(|b| b.len() > CONTENT_BYTES) {
            preview.unsupported = Some(UnsupportedContent::Oversized);
        } else if bytes.as_ref().is_some_and(|b| b.contains(&0)) {
            preview.unsupported = Some(UnsupportedContent::Binary);
        } else if bytes
            .as_ref()
            .is_some_and(|b| std::str::from_utf8(b).is_err())
        {
            preview.unsupported = Some(UnsupportedContent::NonUtf8);
        } else if file.index_status == "?" {
            let bytes = bytes.ok_or_else(expired)?;
            let text = std::str::from_utf8(&bytes).map_err(|_| expired())?;
            if !text.is_empty() {
                let count = text.split_terminator('\n').count();
                preview.rows.push(DiffRow {
                    kind: RowKind::Hunk,
                    old_line: None,
                    new_line: None,
                    text: format!("@@ -0,0 +1,{count} @@"),
                });
            }
            for (index, line) in text.split_terminator('\n').enumerate() {
                preview.rows.push(DiffRow {
                    kind: RowKind::Added,
                    old_line: None,
                    new_line: Some(u32::try_from(index + 1).map_err(|_| expired())?),
                    text: line.into(),
                });
            }
            if !text.is_empty() && !text.ends_with('\n') {
                preview.rows.push(DiffRow {
                    kind: RowKind::NoNewline,
                    old_line: None,
                    new_line: None,
                    text: "\\ No newline at end of file".into(),
                });
            }
            if text.is_empty() {
                preview.note = Some("Empty new file.".into());
            }
        } else {
            let diff = effects.diff(root, &comparison.head, &file.path)?;
            if diff
                .split(|b| *b == b'\n')
                .take_while(|line| !line.starts_with(b"@@ "))
                .any(|line| line.starts_with(b"Binary files ") || line == b"GIT binary patch")
            {
                preview.unsupported = Some(UnsupportedContent::Binary);
            } else if std::str::from_utf8(&diff).is_err() {
                preview.unsupported = Some(UnsupportedContent::NonUtf8);
            } else {
                preview.rows = parse_diff(&diff)?;
                if preview.rows.is_empty() {
                    let text = std::str::from_utf8(&diff).map_err(|_| expired())?;
                    preview.note = Some(if diff.is_empty() { "No net text change against HEAD. Staged and unstaged changes may cancel." }
                        else if text.lines().any(|line| line.starts_with("new file mode ")) { "Empty added file." }
                        else if text.lines().any(|line| line.starts_with("deleted file mode ")) { "Empty deleted file." }
                        else { "Metadata-only change." }.into());
                }
            }
        }
        if effects.evidence(root, &file.path)? != before {
            return Err(expired());
        }
    }
    if effects.head(root)? != comparison.head || effects.status(root)? != comparison.status {
        return Err(expired());
    }
    bound_preview(target, preview)
}
fn bound_preview(target: &PreviewTarget, mut preview: FilePreview) -> Result<PreviewObservation> {
    let rows = std::mem::take(&mut preview.rows);
    let mut observation = PreviewObservation {
        target: target.clone(),
        phase: InspectionPhase::Ready,
        preview: Some(preview),
        error: None,
    };
    // Include the actual limited label when measuring each complete response.
    observation.preview.as_mut().unwrap().limited =
        Some("Preview limited by the 200-row or 48 KiB response budget.".into());
    let mut accepted = 0;
    for row in &rows {
        if accepted == 200 {
            break;
        }
        observation.preview.as_mut().unwrap().rows.push(row.clone());
        if serde_json::to_vec(&WorkspaceReply::ChangePreview {
            observation: observation.clone(),
        })
        .map_err(RepositoryError::storage)?
        .len()
            > RESPONSE_BYTES
        {
            observation.preview.as_mut().unwrap().rows.pop();
            let mut single_row = observation.clone();
            single_row.preview.as_mut().unwrap().rows = vec![row.clone()];
            if serde_json::to_vec(&WorkspaceReply::ChangePreview {
                observation: single_row,
            })
            .map_err(RepositoryError::storage)?
            .len()
                > RESPONSE_BYTES
            {
                observation.preview.as_mut().unwrap().unsupported =
                    Some(UnsupportedContent::PreviewLimit);
            }
            break;
        }
        accepted += 1;
    }
    let preview = observation.preview.as_mut().unwrap();
    if accepted == rows.len() {
        preview.limited = None;
    } else if accepted == 0 {
        preview.unsupported = Some(UnsupportedContent::PreviewLimit);
    }
    while serde_json::to_vec(&WorkspaceReply::ChangePreview {
        observation: observation.clone(),
    })
    .map_err(RepositoryError::storage)?
    .len()
        > RESPONSE_BYTES
    {
        if observation.preview.as_mut().unwrap().rows.pop().is_none() {
            return Err(invalid("Preview metadata exceeds its response budget."));
        }
    }
    Ok(observation)
}

fn bound_failed_preview(observation: &mut PreviewObservation) {
    // Errors can expand under JSON escaping. Re-budget retained rows with the
    // actual failure envelope rather than assuming a fixed diagnostic reserve.
    if serde_json::to_vec(&WorkspaceReply::ChangePreview {
        observation: observation.clone(),
    })
    .is_ok_and(|bytes| bytes.len() <= RESPONSE_BYTES)
    {
        return;
    }
    if let Some(preview) = observation.preview.as_mut() {
        preview.limited = Some(
            "Preview limited by the 48 KiB response budget, including failure details.".into(),
        );
    }
    while serde_json::to_vec(&WorkspaceReply::ChangePreview {
        observation: observation.clone(),
    })
    .is_ok_and(|bytes| bytes.len() > RESPONSE_BYTES)
    {
        if let Some(preview) = observation.preview.as_mut()
            && !preview.rows.is_empty()
        {
            preview.rows.pop();
        } else {
            break;
        }
    }
}

pub(super) fn process_inspection(
    shared: &Mutex<OwnerState>,
    effects: &mut impl InspectionEffects,
) -> bool {
    let job = {
        let Ok(mut state) = shared.lock() else {
            return false;
        };
        let Some(job) = state.inspections.pending.pop_front() else {
            return false;
        };
        if let Ok(record) = state.inspections.record(job.workspace_id()) {
            match &job {
                Job::Refresh { .. } => record.phase = InspectionPhase::Running,
                Job::Preview { target, .. } => {
                    if let Some(p) = record.preview.as_mut().filter(|p| p.target == *target) {
                        p.phase = InspectionPhase::Running;
                    }
                }
            }
        }
        state.inspections.active = Some(job.clone());
        let _ = state.change();
        job
    };
    // All effects run after releasing the owner/storage lock.
    match &job {
        Job::Refresh {
            workspace,
            repository,
        } => {
            let result = capture(workspace, repository, effects);
            if let Ok(mut state) = shared.lock() {
                if let Ok(record) = state.inspections.record(workspace.id) {
                    match result {
                        Ok(mut comparison) => {
                            if let Some(old) = &record.comparison {
                                for file in &mut comparison.files {
                                    if let Some(previous) =
                                        old.files.iter().find(|f| f.path == file.path)
                                    {
                                        file.id = previous.id;
                                    }
                                }
                            }
                            record.comparison = Some(comparison);
                            record.phase = InspectionPhase::Ready;
                            record.error = None;
                        }
                        Err(error) => {
                            record.phase = InspectionPhase::Failed;
                            record.error = Some(error);
                        }
                    }
                }
                state.inspections.active = None;
                let _ = state.change();
            }
        }
        Job::Preview {
            workspace,
            repository,
            target,
            comparison,
            file,
        } => {
            let result = acquire_preview(workspace, repository, target, comparison, file, effects);
            if let Ok(mut state) = shared.lock() {
                if let Ok(record) = state.inspections.record(workspace.id)
                    && let Some(p) = record.preview.as_mut().filter(|p| p.target == *target)
                {
                    match result {
                        Ok(observation) => *p = observation,
                        Err(error) => {
                            p.phase = InspectionPhase::Failed;
                            p.error = Some(error);
                            bound_failed_preview(p);
                        }
                    }
                }
                state.inspections.active = None;
                let _ = state.change();
            }
        }
    }
    true
}

impl Workspaces {
    pub(super) fn inspection_request(&self, request: WorkspaceRequest) -> Result<WorkspaceReply> {
        let mut state = self.state()?;
        let id = match &request {
            WorkspaceRequest::Changes { workspace_id, .. } => *workspace_id,
            WorkspaceRequest::ChangePreview { target, .. } => target.workspace_id,
            _ => unreachable!(),
        };
        let workspace = state
            .store()?
            .workspace(id)?
            .ok_or_else(|| RepositoryError::new(ErrorCode::NotFound, "Workspace not found."))?;
        if state.workspace_observation(workspace.clone()).status != CreationStatus::Ready {
            return Err(RepositoryError::new(
                ErrorCode::Unavailable,
                "Inspect changes after the workspace is ready.",
            ));
        }
        let repository = state
            .store()?
            .repository(workspace.repository_id)?
            .ok_or_else(|| RepositoryError::new(ErrorCode::NotFound, "Repository not found."))?;
        let reply = match request {
            WorkspaceRequest::Changes {
                refresh,
                comparison_id,
                after,
                ..
            } => state
                .inspections
                .changes(workspace, repository, refresh, comparison_id, after),
            WorkspaceRequest::ChangePreview { target, load } => state
                .inspections
                .preview(workspace, repository, target, load),
            _ => unreachable!(),
        };
        drop(state);
        self.notify();
        reply
    }
}

#[cfg(test)]
mod tests;
