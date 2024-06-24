//! # Macros
//!
//! Useful macros for Ledger crate.

/// Adds a new item to a `Vec` member, which guarded by a `Cell`.
#[macro_export]
macro_rules! add_item_to_vec {
    ($member:expr, $item:expr) => {
        // Update item list.
        let mut items = $member.take();
        items.push($item);

        // Commit new change.
        $member.set(items);
    };
}
/// Returns items of a `Vec` member, which guarded by a `Cell`.
#[macro_export]
macro_rules! return_vec_item {
    ($member:expr) => {
        let items = $member.take();
        $member.set(items.clone());

        return items;
    };
}

/// Updates an item, which guarded by a `Cell`.
#[macro_export]
macro_rules! update_item {
    ($member:expr, $item:expr) => {
        $member.set($item);
    };
}

/// Assigns an item from member to given assignee, which guarded by a `Cell`.
#[macro_export]
macro_rules! get_item {
    ($member:expr, $assignee:expr) => {
        $assignee = $member.take();
        $member.set($assignee.clone());
    };
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    /// Temporary struct for macro testing.
    #[derive(Default)]
    struct Test {
        pub vec_member_1: Cell<Vec<isize>>,
        pub int_member_1: Cell<isize>,
    }

    #[test]
    fn add_get_item_to_vec() {
        let strct = Test::default();

        let mut items: Vec<isize>;
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

        let mut item: isize;
        get_item!(strct.int_member_1, item);
        assert_eq!(item, isize::default());

        // Don't you dare.
        assert_ne!(isize::default(), 0x45);

        update_item!(strct.int_member_1, 0x45);

        get_item!(strct.int_member_1, item);
        assert_eq!(item, 0x45);
    }
}
