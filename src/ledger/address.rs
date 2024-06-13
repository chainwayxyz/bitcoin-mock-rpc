//! # Address
//!
//! This crate provides address related ledger interfaces.

use super::Ledger;
use crate::{add_item, get_item};
use bitcoin::{Address, Network, XOnlyPublicKey};
use secp256k1::{rand, Keypair, PublicKey, Secp256k1, SecretKey};

/// User's keys and generated address.
#[derive(Clone, Debug)]
pub struct UserCredential {
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
    ) -> UserCredential {
        let credentials = UserCredential {
            secret_key,
            public_key,
            x_only_public_key,
            address,
        };

        add_item!(self.credentials, credentials.clone());

        credentials
    }
    /// Returns secret/public key + address list of the user.
    pub fn _get_address(&self) -> Vec<UserCredential> {
        get_item!(self.credentials);
    }

    /// Generates a random secret/public key pair and creates a new address from
    /// them.
    pub fn generate_address(&self) -> UserCredential {
        let secp = Secp256k1::new();
        // let secret_key = PrivateKey::generate(Network::Regtest);
        // let public_key = PublicKey::from_private_key(&secp, &secret_key);
        let (secret_key, _public_key) = secp.generate_keypair(&mut rand::thread_rng());
        let keypair = Keypair::from_secret_key(&secp, &secret_key);
        let (x_only_public_key, _parity) = XOnlyPublicKey::from_keypair(&keypair);

        let address = Address::p2tr(&secp, x_only_public_key, None, Network::Regtest);

        self.add_address(
            keypair.secret_key(),
            keypair.public_key(),
            keypair.x_only_public_key().0,
            address,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::ledger::Ledger;
    use bitcoin::AddressType;

    #[test]
    fn addresses() {
        let ledger = Ledger::new();
        assert_eq!(ledger.credentials.take().len(), 0);

        ledger.generate_address();
        let credentials = ledger.credentials.take();
        assert_eq!(credentials.len(), 1);

        let credential = credentials.get(0).unwrap().to_owned();

        assert_eq!(
            credential.address.address_type().unwrap(),
            AddressType::P2tr
        );
        assert!(credential
            .address
            .as_unchecked()
            .is_valid_for_network(bitcoin::Network::Regtest));
        // assert!(credential
        //     .address
        //     .is_related_to_xonly_pubkey(&credential.x_only_public_key));
        // assert!(credential
        //     .address
        //     .is_related_to_pubkey(&credential.public_key.into()));
    }
}
