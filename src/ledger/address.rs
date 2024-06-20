//! # Address
//!
//! This crate provides address related ledger interfaces.

use std::str::FromStr;

use super::Ledger;
use crate::{add_item, get_item};
use bitcoin::{
    key::UntweakedPublicKey,
    opcodes::OP_TRUE,
    taproot::{LeafVersion, TaprootBuilder},
    Address, Network, ScriptBuf, Witness, WitnessProgram, XOnlyPublicKey,
};
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
    pub fn add_credential(
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
    pub fn _get_credentials(&self) -> Vec<UserCredential> {
        get_item!(self.credentials);
    }

    /// Generates a random secret/public key pair and creates a new Bicoin
    /// address from them.
    pub fn generate_credential(&self) -> UserCredential {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());
        let keypair = Keypair::from_secret_key(&secp, &secret_key);
        let (x_only_public_key, _parity) = XOnlyPublicKey::from_keypair(&keypair);

        let address = Address::p2tr(&secp, x_only_public_key, None, Network::Regtest);

        self.add_credential(secret_key, public_key, x_only_public_key, address)
    }

    pub fn create_witness() -> (WitnessProgram, Witness) {
        let secp = bitcoin::secp256k1::Secp256k1::new();
        let internal_key = UntweakedPublicKey::from(
            bitcoin::secp256k1::PublicKey::from_str(
                "0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0",
            )
            .unwrap(),
        );

        let mut script = ScriptBuf::new();
        script.push_instruction(bitcoin::script::Instruction::Op(OP_TRUE));

        let taproot_builder = TaprootBuilder::new().add_leaf(0, script.clone()).unwrap();
        let taproot_spend_info = taproot_builder.finalize(&secp, internal_key).unwrap();

        let witness_program =
            WitnessProgram::p2tr(&secp, internal_key, taproot_spend_info.merkle_root());

        let mut control_block_bytes = Vec::new();
        taproot_spend_info
            .control_block(&(script.clone(), LeafVersion::TapScript))
            .unwrap()
            .encode(&mut control_block_bytes)
            .unwrap();

        let mut witness = Witness::new();
        witness.push(script.to_bytes());
        witness.push(control_block_bytes);

        (witness_program, witness)
    }

    /// Creates a Bitcoin address from a witness program.
    pub fn create_address() -> Address {
        let witness_program = Self::create_witness().0;

        Address::from_witness_program(witness_program, bitcoin::Network::Regtest)
    }
}

#[cfg(test)]
mod tests {
    use crate::ledger::Ledger;
    use bitcoin::{key::TapTweak, AddressType};
    use secp256k1::Secp256k1;

    #[test]
    fn addresses() {
        let ledger = Ledger::new();
        assert_eq!(ledger.credentials.take().len(), 0);

        ledger.generate_credential();
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
        assert!(credential.address.is_related_to_xonly_pubkey(
            &credential
                .x_only_public_key
                .tap_tweak(&Secp256k1::new(), None)
                .0
                .into()
        ));
    }
}
