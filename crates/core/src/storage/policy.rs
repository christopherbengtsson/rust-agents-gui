//! Storage decisions tested without SQLite, filesystem or process effects.
pub(super) const ITEM_BYTES: usize = 64 * 1024;
pub(super) const PAGE_BYTES: usize = 48 * 1024;
pub(super) const PAGE_ROWS: usize = 32;

pub(super) fn reusable(ping_ok: bool, tracked_transaction: bool) -> bool {
    ping_ok && !tracked_transaction
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn responsive_worker_with_unresolved_rollback_is_never_reused() {
        assert!(!reusable(true, true));
        assert!(!reusable(false, false));
        assert!(!reusable(false, true));
        assert!(reusable(true, false));
    }
}
