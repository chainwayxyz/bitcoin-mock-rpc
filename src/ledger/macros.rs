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
        for i in 0..items.len() {
            if items[i] == $item {
                items.remove(i);
                break;
            }
        }

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
/// Assigns an item from member to given mutable assignee, which is guarded by a
/// `Cell`.
#[macro_export]
macro_rules! get_mut_item {
    ($member:expr, $assignee:ident) => {
        let mut $assignee = $member.lock().unwrap().take();
        $member.lock().unwrap().set($assignee.clone());
    };
}

/// Adds a new UTXO for an address, which is guarded by a `Cell`.
#[macro_export]
macro_rules! add_utxo_to_address {
    ($member:expr, $address:expr, $utxo:expr) => {
        // Update utxo list.
        let mut address_utxos = $member.lock().unwrap().take();

        address_utxos
            .entry($address)
            .and_modify(|utxos| utxos.push($utxo))
            .or_insert(vec![$utxo]);

        // Commit new change.
        $member.lock().unwrap().set(address_utxos);
    };
}
/// Gets UTXO's for the address, which is guarded by a `Cell`.
#[macro_export]
macro_rules! get_utxos_for_address {
    ($member:expr, $address:expr, $assignee:ident) => {
        let $assignee = $member.lock().unwrap().take();
        $member.lock().unwrap().set($assignee.clone());
        let $assignee = $assignee.get(&$address.clone()).unwrap().to_owned();
    };
}
/// Removes given UTXO from an address, which is guarded by a `Cell`.
#[macro_export]
macro_rules! remove_utxo_from_address {
    ($member:expr, $address:expr, $item:expr) => {
        // Get item list.
        let mut address_utxos = $member.lock().unwrap().take();
        address_utxos.entry($address).and_modify(|address_utxo| {
            // Delete given item.
            for i in 0..address_utxo.len() {
                if address_utxo[i] == $item {
                    address_utxo.remove(i);
                    break;
                }
            }
        });

        // Commit new change.
        $member.lock().unwrap().set(address_utxos);
    };
}

#[cfg(test)]
mod tests {
    use bitcoin::{hashes::Hash, OutPoint, Txid};
    use std::{
        cell::Cell,
        sync::{Arc, Mutex},
    };

    use crate::ledger::Ledger;

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

    #[test]
    fn add_get_remove_utxo_to_address() {
        let ledger = Ledger::new();
        let address = Ledger::generate_credential().address;

        let utxos = [
            OutPoint {
                txid: Txid::all_zeros(),
                vout: 1,
            },
            OutPoint {
                txid: Txid::all_zeros(),
                vout: 2,
            },
            OutPoint {
                txid: Txid::all_zeros(),
                vout: 3,
            },
        ];

        add_utxo_to_address!(ledger.utxos, address.clone(), utxos[0]);
        get_utxos_for_address!(ledger.utxos, address.clone(), get_utxos);
        assert_eq!(get_utxos, vec![utxos[0]]);

        add_utxo_to_address!(ledger.utxos, address.clone(), utxos[1]);
        get_utxos_for_address!(ledger.utxos, address.clone(), get_utxos);
        assert_eq!(get_utxos, vec![utxos[0], utxos[1]]);

        add_utxo_to_address!(ledger.utxos, address.clone(), utxos[2]);
        get_utxos_for_address!(ledger.utxos, address.clone(), get_utxos);
        assert_eq!(get_utxos, vec![utxos[0], utxos[1], utxos[2]]);

        remove_utxo_from_address!(ledger.utxos, address.clone(), utxos[1]);
        get_utxos_for_address!(ledger.utxos, address.clone(), get_utxos);
        assert_eq!(get_utxos, vec![utxos[0], utxos[2]]);

        // Should not change anything.
        remove_utxo_from_address!(ledger.utxos, address.clone(), utxos[1]);
        get_utxos_for_address!(ledger.utxos, address.clone(), get_utxos);
        assert_eq!(get_utxos, vec![utxos[0], utxos[2]]);

        remove_utxo_from_address!(ledger.utxos, address.clone(), utxos[0]);
        get_utxos_for_address!(ledger.utxos, address.clone(), get_utxos);
        assert_eq!(get_utxos, vec![utxos[2]]);

        remove_utxo_from_address!(ledger.utxos, address.clone(), utxos[2]);
        get_utxos_for_address!(ledger.utxos, address.clone(), get_utxos);
        assert_eq!(get_utxos, vec![]);
    }
}
