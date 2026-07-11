//! Pure keyed reconciliation for authoritative clipboard snapshots.

use std::collections::{HashMap, HashSet};

use author_clipboard_shared::types::ClipboardItem;

/// Minimal operations needed to reconcile a rendered list by stable item ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcilePlan {
    /// IDs not present in the new snapshot.
    pub removed: Vec<i64>,
    /// IDs newly introduced by the snapshot.
    pub inserted: Vec<i64>,
    /// Existing IDs whose row position changed.
    pub moved: Vec<(i64, usize)>,
    /// Existing IDs that can retain and rebind their row widget.
    pub retained: Vec<i64>,
}

/// Build a deterministic reconciliation plan from old and new snapshots.
pub fn reconcile(old: &[ClipboardItem], new: &[ClipboardItem]) -> ReconcilePlan {
    let old_positions: HashMap<i64, usize> = old
        .iter()
        .enumerate()
        .map(|(index, item)| (item.id, index))
        .collect();
    let new_ids: HashSet<i64> = new.iter().map(|item| item.id).collect();
    let removed = old
        .iter()
        .filter(|item| !new_ids.contains(&item.id))
        .map(|item| item.id)
        .collect();
    let mut inserted = Vec::new();
    let mut moved = Vec::new();
    let mut retained = Vec::new();
    for (index, item) in new.iter().enumerate() {
        match old_positions.get(&item.id).copied() {
            None => inserted.push(item.id),
            Some(old_index) => {
                retained.push(item.id);
                if old_index != index {
                    moved.push((item.id, index));
                }
            }
        }
    }
    ReconcilePlan {
        removed,
        inserted,
        moved,
        retained,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: i64) -> ClipboardItem {
        let mut item = ClipboardItem::new_text(id.to_string());
        item.id = id;
        item
    }

    #[test]
    fn plans_insert_remove_move_and_retain() {
        let plan = reconcile(&[item(1), item(2), item(3)], &[item(3), item(2), item(4)]);
        assert_eq!(plan.removed, vec![1]);
        assert_eq!(plan.inserted, vec![4]);
        assert_eq!(plan.moved, vec![(3, 0)]);
        assert_eq!(plan.retained, vec![3, 2]);
    }

    #[test]
    fn thousand_item_single_insert_reuses_every_existing_row() {
        let old: Vec<_> = (0..1_000).map(item).collect();
        let mut new = vec![item(2_000)];
        new.extend(old.iter().cloned());
        let plan = reconcile(&old, &new);
        assert_eq!(plan.inserted, vec![2_000]);
        assert_eq!(plan.retained.len(), 1_000);
        assert!(plan.removed.is_empty());
    }
}
