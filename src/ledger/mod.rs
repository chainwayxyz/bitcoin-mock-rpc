//! # Bitcoin Ledger
//!
//! Mock Bitcoin ledger.

use bitcoin::{Address, TxOut};

/// Mock Bitcoin ledger structure.
#[derive(Clone, Default)]
pub struct Ledger {
    /// User's addresses.
    pub addresses: Vec<Address>,
    /// User's unspent transaction outputs.
    pub utxos: Vec<TxOut>,
}

impl Ledger {
    /// Creates a new empty ledger.
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Adds a new UTXO to user's UTXO's.
    pub fn add_utxo(&self, utxo: TxOut) -> Self {
        let mut ledger = self.clone().to_owned();

        ledger.utxos.push(utxo);

        ledger
    }
    /// Returns UTXO's of the user.
    pub fn get_utxos(&self) -> Vec<TxOut> {
        self.utxos.clone()
    }

    /// Adds a new address for user.
    pub fn add_address(&self, address: Address) -> Self {
        let mut ledger = self.clone().to_owned();

        ledger.addresses.push(address);

        ledger
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::{Amount, Network, TxOut, XOnlyPublicKey};
    use secp256k1::Secp256k1;

    #[test]
    fn new() {
        let _should_not_panic = Ledger::new();
    }

    #[test]
    fn add_utxo() {
        let mut ledger = Ledger::new();

        assert_eq!(ledger.utxos.len(), 0);
        assert_eq!(ledger.addresses.len(), 0);

        // Generate a random address.
        let secp = Secp256k1::new();
        let xonly_public_key = XOnlyPublicKey::from_slice(&[
            0x78u8, 0x19u8, 0x90u8, 0xd7u8, 0xe2u8, 0x11u8, 0x8cu8, 0xc3u8, 0x61u8, 0xa9u8, 0x3au8,
            0x6fu8, 0xccu8, 0x54u8, 0xceu8, 0x61u8, 0x1du8, 0x6du8, 0xf3u8, 0x81u8, 0x68u8, 0xd6u8,
            0xb1u8, 0xedu8, 0xfbu8, 0x55u8, 0x65u8, 0x35u8, 0xf2u8, 0x20u8, 0x0cu8, 0x4b,
        ])
        .unwrap();
        let address = Address::p2tr(&secp, xonly_public_key, None, Network::Regtest);
        ledger = ledger.add_address(address);
        assert_eq!(ledger.addresses.len(), 1);

        // Insert a dummy UTXO.
        let utxo = TxOut {
            value: Amount::from_sat(0x45),
            script_pubkey: ledger.addresses[0].script_pubkey(),
        };
        ledger = ledger.add_utxo(utxo);

        assert_eq!(ledger.utxos.len(), 1);
        assert_eq!(ledger.utxos[0].value, Amount::from_sat(0x45));
    }
}
