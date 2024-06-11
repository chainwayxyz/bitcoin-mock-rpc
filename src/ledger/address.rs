//! # Address
//!
//! This crate provides address related ledger interfaces.

use super::Ledger;
use crate::{add_item, get_item};
use bitcoin::{Address, Network, XOnlyPublicKey};
use secp256k1::{rand, Keypair, PublicKey, Secp256k1, SecretKey};

/// User's keys and generated address.
#[derive(Clone, Debug)]
pub struct UserAddress {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
    pub x_only_public_key: XOnlyPublicKey,
    pub address: Address,
}

impl Ledger {
    /// Adds a new secret/public key + address for the user.
    pub fn add_address(
        &self,
        secret_key: SecretKey,
        public_key: PublicKey,
        x_only_public_key: XOnlyPublicKey,
        address: Address,
    ) -> UserAddress {
        let addresses = UserAddress {
            secret_key,
            public_key,
            x_only_public_key,
            address,
        };

        add_item!(self.addresses, addresses.clone());

        addresses
    }

    /// Generates a random secret/public key pair and creates a new address from
    /// them.
    pub fn generate_address(&self) -> UserAddress {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());
        let (x_only_public_key, _parity) =
            XOnlyPublicKey::from_keypair(&Keypair::from_secret_key(&secp, &secret_key));

        let address = Address::p2tr(&secp, x_only_public_key, None, Network::Regtest);

        self.add_address(secret_key, public_key, x_only_public_key, address)
    }

    /// Returns secret/public key + address list of the user.
    pub fn _get_address(&self) -> Vec<UserAddress> {
        get_item!(self.addresses);
    }
}

#[cfg(test)]
mod tests {
    use crate::ledger::Ledger;
    use bitcoin::AddressType;

    #[test]
    fn addresses() {
        let ledger = Ledger::new();
        assert_eq!(ledger.addresses.take().len(), 0);

        ledger.generate_address();
        let addresses = ledger.addresses.take();
        assert_eq!(addresses.len(), 1);

        let address = addresses.get(0).unwrap().to_owned();

        assert_eq!(address.address.address_type().unwrap(), AddressType::P2tr);
        assert!(address
            .address
            .as_unchecked()
            .is_valid_for_network(bitcoin::Network::Regtest));
        // assert!(address.address.is_related_to_xonly_pubkey(&address.x_only_public_key));
    }
}
