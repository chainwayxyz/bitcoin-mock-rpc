//! # Address
//!
//! This crate provides address related ledger interfaces.

use super::Ledger;
use bitcoin::{
    opcodes::OP_TRUE,
    taproot::{LeafVersion, TaprootBuilder},
    Address, Network, ScriptBuf, Witness, WitnessProgram, XOnlyPublicKey,
};
use secp256k1::{rand, Keypair, PublicKey, Secp256k1, SecretKey};

/// User's keys and generated address.
#[derive(Clone, Debug, PartialEq)]
pub struct UserCredential {
    pub secp: Secp256k1<secp256k1::All>,
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
    pub x_only_public_key: XOnlyPublicKey,
    pub address: Address,
    pub witness: Option<Witness>,
    pub witness_program: Option<WitnessProgram>,
}

impl UserCredential {
    /// Creates a new `UserCredential` with random keys. Bitcoin address has the
    /// type of "p2tr".
    pub fn new() -> Self {
        let secp = Secp256k1::new();

        let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());
        tracing::trace!(
            "New secret/public key pair: {:?} {:?}",
            secret_key,
            public_key
        );

        let keypair = Keypair::from_secret_key(&secp, &secret_key);
        let (x_only_public_key, _parity) = XOnlyPublicKey::from_keypair(&keypair);
        tracing::trace!("New x-only public key: {:?}", x_only_public_key);

        let address = Address::p2tr(&secp, x_only_public_key, None, Network::Regtest);
        tracing::trace!("New bitcoin address: {:?}", address);

        Self {
            secp,
            secret_key,
            public_key,
            x_only_public_key,
            address,
            witness: None,
            witness_program: None,
        }
    }
}

impl Default for UserCredential {
    fn default() -> Self {
        Self::new()
    }
}

impl Ledger {
    /// Generates a random secret/public key pair and creates a new Bicoin
    /// address from them.
    #[tracing::instrument]
    pub fn generate_credential() -> UserCredential {
        UserCredential::new()
    }
    /// Generates a Bitcoin credentials from a witness program.
    #[tracing::instrument]
    pub fn generate_credential_from_witness() -> UserCredential {
        let mut credential = Ledger::generate_credential();

        Ledger::create_witness(&mut credential);

        credential.address = Address::from_witness_program(
            credential.witness_program.unwrap(),
            bitcoin::Network::Regtest,
        );

        credential
    }
    /// Generates the constant Bitcoin credentials from a witness program.
    #[tracing::instrument]
    pub fn get_constant_credential_from_witness() -> UserCredential {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&[0x45; 32]).unwrap();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let x_only_public_key =
            XOnlyPublicKey::from_keypair(&Keypair::from_secret_key(&secp, &secret_key)).0;
        let address = Address::p2tr(&secp, x_only_public_key, None, Network::Regtest);

        let mut credential = UserCredential {
            secp,
            secret_key,
            public_key,
            x_only_public_key,
            address,
            witness: None,
            witness_program: None,
        };
        tracing::trace!("Constant credentials: {credential:?}");

        Ledger::create_witness(&mut credential);

        credential.address = Address::from_witness_program(
            credential.witness_program.unwrap(),
            bitcoin::Network::Regtest,
        );

        credential
    }

    /// Generates a random Bicoin address.
    pub fn _generate_address() -> Address {
        UserCredential::new().address
    }
    /// Generates a Bitcoin address from a witness program.
    pub fn generate_address_from_witness() -> Address {
        Ledger::generate_credential_from_witness().address
    }

    /// Creates a witness for the given secret/public key pair.
    pub fn create_witness(credential: &mut UserCredential) {
        let mut script = ScriptBuf::new();
        script.push_instruction(bitcoin::script::Instruction::Op(OP_TRUE));

        let taproot_builder = TaprootBuilder::new().add_leaf(0, script.clone()).unwrap();
        let taproot_spend_info = taproot_builder
            .finalize(&credential.secp, credential.x_only_public_key)
            .unwrap();
        tracing::trace!(
            "Taproot spend info for the new witness: {:?}",
            taproot_spend_info
        );

        let witness_program = WitnessProgram::p2tr(
            &credential.secp,
            credential.x_only_public_key,
            taproot_spend_info.merkle_root(),
        );
        tracing::trace!("New witness program: {:?}", witness_program);

        let mut control_block_bytes = Vec::new();
        taproot_spend_info
            .control_block(&(script.clone(), LeafVersion::TapScript))
            .unwrap()
            .encode(&mut control_block_bytes)
            .unwrap();

        let mut witness = Witness::new();
        witness.push(script.to_bytes());
        witness.push(control_block_bytes);
        tracing::trace!("New witness: {:?}", witness);

        credential.witness = Some(witness);
        credential.witness_program = Some(witness_program);
    }
}

#[cfg(test)]
mod tests {
    use crate::ledger::Ledger;
    use bitcoin::{key::TapTweak, AddressType};

    #[test]
    fn generate_credentials() {
        let credential = Ledger::generate_credential();
        println!("{:?}", credential.secret_key);

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
                .tap_tweak(&credential.secp, None)
                .0
                .into()
        ));
    }
}
