//! # Macros
//!
//! Useful macros for Ledger crate.

/// Adds a new item to a `Vec` member, which is guarded by a `Cell`.
#[macro_export]
macro_rules! add_item_to_vec {
    ($member:expr, $item:expr) => {
        // Update item list.
        let mut items = $member.lock().unwrap().take();
        items.push($item);

        // Commit new change.
        $member.lock().unwrap().set(items);
    };
}
/// Removes given item from a `Vec` member, which is guarded by a `Cell`.
#[macro_export]
macro_rules! remove_item_from_vec {
    ($member:expr, $item:expr) => {
        // Get item list.
        let mut items = $member.lock().unwrap().take();

        // Delete given item.
        items.retain(|&i| i != $item);

        // Commit new change.
        $member.lock().unwrap().set(items);
    };
}
/// Returns items of a `Vec` member, which is guarded by a `Cell`.
#[macro_export]
macro_rules! return_vec_item {
    ($member:expr) => {
        let items = $member.lock().unwrap().take();
        $member.lock().unwrap().set(items.clone());

        return items;
    };
}

/// Updates an item, which is guarded by a `Cell`.
#[macro_export]
macro_rules! update_item {
    ($member:expr, $item:expr) => {
        $member.lock().unwrap().set($item);
    };
}

/// Assigns an item from member to given assignee, which is guarded by a `Cell`.
#[macro_export]
macro_rules! get_item {
    ($member:expr, $assignee:ident) => {
        let $assignee = $member.lock().unwrap().take();
        $member.lock().unwrap().set($assignee.clone());
    };
}

#[cfg(test)]
mod tests {
    use std::{
        cell::Cell,
        sync::{Arc, Mutex},
    };

    /// Temporary struct for macro testing.
    #[derive(Default)]
    struct Test {
        pub vec_member_1: Arc<Mutex<Cell<Vec<isize>>>>,
        pub int_member_1: Arc<Mutex<Cell<isize>>>,
    }

    #[test]
    fn add_get_item_to_vec() {
        let strct = Test::default();

        get_item!(strct.vec_member_1, items);
        assert_eq!(items.len(), 0);

        add_item_to_vec!(strct.vec_member_1, 0x45);

        get_item!(strct.vec_member_1, items);
        assert_eq!(items.len(), 1);
        assert_eq!(*items.get(0).unwrap(), 0x45);
    }

    #[test]
    fn update_member() {
        let strct = Test::default();

        get_item!(strct.int_member_1, item);
        assert_eq!(item, isize::default());

        // Don't you dare.
        assert_ne!(isize::default(), 0x45);

        update_item!(strct.int_member_1, 0x45);

        get_item!(strct.int_member_1, item);
        assert_eq!(item, 0x45);
    }

    #[test]
    fn remove_item_from_vec() {
        let strct = Test::default();

        add_item_to_vec!(strct.vec_member_1, 0x45);
        add_item_to_vec!(strct.vec_member_1, 0x1F);
        add_item_to_vec!(strct.vec_member_1, 0x100);

        get_item!(strct.vec_member_1, items);
        assert_eq!(items.len(), 3);
        assert_eq!(*items.get(0).unwrap(), 0x45);
        assert_eq!(*items.get(1).unwrap(), 0x1F);
        assert_eq!(*items.get(2).unwrap(), 0x100);

        remove_item_from_vec!(strct.vec_member_1, 0x1F);

        get_item!(strct.vec_member_1, items);
        assert_eq!(items.len(), 2);
        assert_eq!(*items.get(0).unwrap(), 0x45);
        assert_eq!(*items.get(1).unwrap(), 0x100);
    }
}
