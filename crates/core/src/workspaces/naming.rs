//! Human labels are permanently reserved; UUIDs remain internal identity.
use super::*;
use std::collections::HashSet;
#[path = "naming/catalog.rs"]
mod catalog;

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

/// A stable permutation of the catalogue at each numeric tier. The persisted
/// cursor visits one candidate at a time; it never reloads historical names.
pub(crate) fn candidate(cursor: u64, seed: u128) -> Result<String> {
    let count = u64::try_from(catalog::PLAYERS.len()).map_err(RepositoryError::storage)?;
    let tier = cursor
        .checked_div(count)
        .and_then(|n| n.checked_add(1))
        .ok_or_else(|| RepositoryError::new(ErrorCode::Capacity, "Allocation cursor exhausted"))?;
    let offset = u64::try_from(seed % u128::from(count)).map_err(RepositoryError::storage)?;
    let index =
        usize::try_from((cursor % count + offset) % count).map_err(RepositoryError::storage)?;
    Ok(numbered(catalog::PLAYERS[index], tier))
}
pub(crate) fn numbered(base: &str, tier: u64) -> String {
    if tier == 1 {
        base.into()
    } else {
        format!("{base}-{tier}")
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Occupancy {
    Available,
    RepositoryFolder,
    WorkspaceName,
}
#[derive(Default)]
pub(crate) struct Occupied {
    pub names: HashSet<String>,
    pub worktrees: Vec<PathBuf>,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_and_numeric_continuation_are_bounded_and_permanent() {
        assert_eq!(catalog::PLAYERS.len(), 500);
        assert_eq!(catalog::PLAYERS.iter().collect::<HashSet<_>>().len(), 500);
        assert_eq!(candidate(0, 0).unwrap(), catalog::PLAYERS[0]);
        assert_eq!(candidate(499, 0).unwrap(), catalog::PLAYERS[499]);
        assert_eq!(
            candidate(500, 0).unwrap(),
            format!("{}-2", catalog::PLAYERS[0])
        );
        assert_eq!(
            candidate(1_000_000, 0).unwrap(),
            format!("{}-2001", catalog::PLAYERS[0])
        );
        for name in catalog::PLAYERS {
            assert_eq!(slug(name), *name);
        }
    }
}
