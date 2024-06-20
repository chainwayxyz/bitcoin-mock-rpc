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
/// Updates an item, guarded by a `Cell`.
#[macro_export]
macro_rules! update_item {
    ($member:expr, $item:expr) => {
        // Update item list.
        let mut mut_item = $member.take();
        mut_item = $item;

        // Commit new change.
        $member.set(mut_item);
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
/// Assigns an item from member to given assignee, guarded by a `Cell`.
#[macro_export]
macro_rules! assign_item {
    ($member:expr, $assignee:expr) => {
        $assignee = $member.take();
        $member.set($assignee.clone());
    };
}
