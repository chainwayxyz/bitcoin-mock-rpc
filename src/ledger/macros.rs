//! # Macros
//!
//! Useful macros for Ledger crate.

/// Adds a new item to a `Vec` member, guarded by a `Cell`.
#[macro_export]
macro_rules! add_item {
    ($member:expr, $item:expr) => {
        // Update item list.
        let mut items = $member.take();
        items.push($item);

        // Commit new change.
        $member.set(items);
    };
}
/// Returns item `Vec` of a member, guarded by a `Cell`.
#[macro_export]
macro_rules! get_item {
    ($member:expr) => {
        let items = $member.take();
        $member.set(items.clone());

        return items;
    };
}
