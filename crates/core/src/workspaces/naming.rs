//! Human labels are permanently reserved; UUIDs remain internal identity.
use super::*;
use std::collections::HashSet;
mod catalog;

#[derive(Default)]
pub(crate) struct Naming {
    pub folder: Option<String>,
    pub folders: HashSet<String>,
    pub names: HashSet<String>,
}
#[derive(Default)]
pub(crate) struct Occupied {
    pub names: HashSet<String>,
    pub worktrees: Vec<PathBuf>,
}

pub(crate) fn slug(value: &str) -> String {
    let mut result = String::new();
    for c in value.chars() {
        if c.is_ascii_alphanumeric() {
            if result.len() >= 64 {
                break;
            }
            result.push(c.to_ascii_lowercase());
        } else if !result.is_empty() && !result.ends_with('-') {
            result.push('-');
        }
    }
    let result = result.trim_end_matches('-');
    if result.is_empty() {
        "repository".into()
    } else {
        result.into()
    }
}

pub(crate) fn repository_folder(base: &str, used: &HashSet<String>) -> Result<String> {
    let base = slug(base);
    for tier in 1..=u32::MAX {
        let candidate = numbered(&base, tier);
        if !used.contains(&candidate) {
            return Ok(candidate);
        }
    }
    Err(RepositoryError::new(
        ErrorCode::Capacity,
        "Repository folder numbering is exhausted.",
    ))
}
fn numbered(base: &str, tier: u32) -> String {
    if tier == 1 {
        base.into()
    } else {
        format!("{base}-{tier}")
    }
}

pub(crate) fn workspace_name(
    used: &HashSet<String>,
    occupied: &Occupied,
    directory: &Path,
    random: u128,
) -> Result<String> {
    choose_name(catalog::PLAYERS, used, occupied, directory, random)
}
fn choose_name(
    pool: &[&str],
    used: &HashSet<String>,
    occupied: &Occupied,
    directory: &Path,
    random: u128,
) -> Result<String> {
    if occupied
        .worktrees
        .iter()
        .any(|path| directory.starts_with(path))
    {
        return Err(RepositoryError::new(
            ErrorCode::Conflict,
            "A registered Git worktree occupies this managed repository folder or its ancestor. Restore or move that worktree registration, then reconcile; choosing another player cannot resolve this location conflict.",
        ));
    }
    for tier in 1..=u32::MAX {
        let available: Vec<String> = pool
            .iter()
            .map(|base| numbered(base, tier))
            .filter(|name| {
                !used.contains(name)
                    && !occupied.names.contains(name)
                    && !occupied.worktrees.iter().any(|path| {
                        let wrapper = directory.join(name);
                        path.starts_with(&wrapper) || wrapper.starts_with(path)
                    })
            })
            .collect();
        if !available.is_empty() {
            let index = usize::try_from(random % available.len() as u128)
                .map_err(RepositoryError::storage)?;
            return Ok(available[index].clone());
        }
    }
    Err(RepositoryError::new(
        ErrorCode::Capacity,
        "Workspace name numbering is exhausted.",
    ))
}

// Only the exact historical '-<this workspace UUID>' suffix is hidden. Branches
// and physical folders are retained byte-for-byte; migration persists labels.
pub(crate) fn legacy_label(workspace: &Workspace, used: &HashSet<String>) -> Result<String> {
    let old = workspace.name.as_deref().unwrap_or("workspace");
    let suffix = format!("-{}", workspace.id.0);
    repository_folder(old.strip_suffix(&suffix).unwrap_or(old), used)
}

pub(crate) fn migrate_labels(mut records: Vec<Workspace>) -> Result<Vec<Workspace>> {
    records.sort_by_key(|w| (w.repository_id, w.id));
    let mut used: std::collections::HashMap<RepositoryId, HashSet<String>> = Default::default();
    for workspace in &mut records {
        let names = used.entry(workspace.repository_id).or_default();
        let label = legacy_label(workspace, names)?;
        names.insert(label.clone());
        workspace.name = Some(label);
        workspace.legacy = true;
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_is_unique_safe_and_all_entries_can_be_selected() {
        let pool = catalog::PLAYERS;
        assert_eq!(pool.len(), 500);
        assert_eq!(pool.iter().collect::<HashSet<_>>().len(), 500);
        for (i, name) in pool.iter().enumerate() {
            assert_eq!(slug(name), *name);
            assert_eq!(
                choose_name(
                    pool,
                    &HashSet::new(),
                    &Occupied::default(),
                    Path::new("/state/repo"),
                    i as u128
                )
                .unwrap(),
                *name
            );
        }
    }
    #[test]
    fn full_catalog_reservations_use_numbers_and_legacy_migration_preserves_artifacts() {
        let used: HashSet<String> = catalog::PLAYERS.iter().map(|v| (*v).into()).collect();
        let name = workspace_name(&used, &Occupied::default(), Path::new("/repo"), 499).unwrap();
        assert_eq!(name, format!("{}-2", catalog::PLAYERS[499]));
        let repository = RepositoryId(Uuid::nil());
        let records: Vec<Workspace> = (0..3)
            .map(|i| {
                let mut w =
                    allocate(repository, ActionId(Uuid::nil()), Path::new("/state")).unwrap();
                w.id = WorkspaceId(Uuid::from_u128(i));
                w.name = Some(format!("kyoto-{}", w.id.0));
                w.branch = Some(format!("workspace/{}", w.name.as_deref().unwrap()));
                w.path = Some(format!(
                    "/state/workspaces/{}/{}",
                    repository.0,
                    w.name.as_deref().unwrap()
                ));
                w
            })
            .collect();
        let migrated = migrate_labels(records.clone()).unwrap();
        assert_eq!(
            migrated
                .iter()
                .map(|w| w.name.as_deref().unwrap())
                .collect::<Vec<_>>(),
            ["kyoto", "kyoto-2", "kyoto-3"]
        );
        for (old, new) in records.iter().zip(&migrated) {
            assert_eq!(old.id, new.id);
            assert_eq!(old.branch, new.branch);
            assert_eq!(old.path, new.path);
            assert!(new.legacy);
        }
    }
    #[test]
    fn lowest_tier_and_permanent_history_reservations_precede_numbering() {
        let pool = ["marta", "pele"];
        let mut used = HashSet::from(["marta".into()]);
        let directory = Path::new("/state/repo");
        assert_eq!(
            choose_name(&pool, &used, &Occupied::default(), directory, 0).unwrap(),
            "pele"
        );
        used.extend(["pele".into(), "marta-2".into()]);
        assert_eq!(
            choose_name(&pool, &used, &Occupied::default(), directory, 0).unwrap(),
            "pele-2"
        );
        let occupied = Occupied {
            names: HashSet::from(["pele-2".into(), "marta-3".into()]),
            worktrees: vec![],
        };
        assert_eq!(
            choose_name(&pool, &used, &occupied, directory, 0).unwrap(),
            "pele-3"
        );
        assert_eq!(
            choose_name(&pool, &HashSet::new(), &Occupied::default(), directory, 0).unwrap(),
            "marta"
        );
    }
    #[test]
    fn registered_ancestor_conflicts_fail_before_trying_name_tiers() {
        for path in ["/state/repo", "/state"] {
            let occupied = Occupied {
                names: Default::default(),
                worktrees: vec![PathBuf::from(path)],
            };
            let error = choose_name(
                &["marta"],
                &HashSet::new(),
                &occupied,
                Path::new("/state/repo"),
                0,
            )
            .unwrap_err();
            assert_eq!(error.code, ErrorCode::Conflict);
            assert!(error.message.contains("ancestor"));
        }
    }
    #[test]
    fn foreign_refs_folders_and_registered_worktree_ancestors_are_skipped() {
        let occupied = Occupied {
            names: HashSet::from(["pele".into()]),
            worktrees: vec![PathBuf::from("/state/repo/marta/checkout")],
        };
        assert_eq!(
            choose_name(
                &["pele", "marta"],
                &HashSet::new(),
                &occupied,
                Path::new("/state/repo"),
                0
            )
            .unwrap(),
            "pele-2"
        );
        assert_eq!(
            repository_folder(
                "My.Repository",
                &HashSet::from(["my-repository".into(), "my-repository-2".into()])
            )
            .unwrap(),
            "my-repository-3"
        );
        assert_eq!(slug("💚"), "repository");
    }
}
