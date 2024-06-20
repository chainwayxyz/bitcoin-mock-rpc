//! # Address
//!
//! This crate provides address related ledger interfaces.

use super::Ledger;
use crate::{add_item, get_item};
use bitcoin::{
    opcodes::OP_TRUE,
    taproot::{LeafVersion, TaprootBuilder},
    Address, Network, ScriptBuf, Witness, WitnessProgram, XOnlyPublicKey,
};
use secp256k1::{rand, Keypair, PublicKey, Secp256k1, SecretKey};

/// User's keys and generated address.
#[derive(Clone, Debug)]
pub struct UserCredential {
    secp: Secp256k1<secp256k1::All>,
    secret_key: SecretKey,
    public_key: PublicKey,
    x_only_public_key: XOnlyPublicKey,
    pub address: Address,
    pub witness: Option<Witness>,
    witness_program: Option<WitnessProgram>,
}

impl UserCredential {
    /// Creates a new `UserCredential` with random keys.
    pub fn new() -> Self {
        let secp = Secp256k1::new();

        let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());

        let keypair = Keypair::from_secret_key(&secp, &secret_key);
        let (x_only_public_key, _parity) = XOnlyPublicKey::from_keypair(&keypair);

        let address = Address::p2tr(&secp, x_only_public_key, None, Network::Regtest);

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

impl Ledger {
    /// Adds a new secret/public key + address for the user.
    pub fn add_credential(&self, credential: UserCredential) -> UserCredential {
        add_item!(self.credentials, credential.clone());

        credential
    }
    /// Returns secret/public key + address list of the user.
    pub fn _get_credentials(&self) -> Vec<UserCredential> {
        get_item!(self.credentials);
    }

    /// Generates a random secret/public key pair and creates a new Bicoin
    /// address from them.
    pub fn generate_credential(&self) -> UserCredential {
        let credential = UserCredential::new();

        self.add_credential(credential)
    }

    pub fn create_witness(&self) -> UserCredential {
        let mut credential = self.generate_credential();

        let mut script = ScriptBuf::new();
        script.push_instruction(bitcoin::script::Instruction::Op(OP_TRUE));

        let taproot_builder = TaprootBuilder::new().add_leaf(0, script.clone()).unwrap();
        let taproot_spend_info = taproot_builder
            .finalize(&credential.secp, credential.x_only_public_key)
            .unwrap();

        let witness_program = WitnessProgram::p2tr(
            &credential.secp,
            credential.x_only_public_key,
            taproot_spend_info.merkle_root(),
        );

        let mut control_block_bytes = Vec::new();
        taproot_spend_info
            .control_block(&(script.clone(), LeafVersion::TapScript))
            .unwrap()
            .encode(&mut control_block_bytes)
            .unwrap();

        let mut witness = Witness::new();
        witness.push(script.to_bytes());
        witness.push(control_block_bytes);

        credential.witness = Some(witness);
        credential.witness_program = Some(witness_program);

        credential
    }

    /// Creates a Bitcoin address from a witness program.
    pub fn create_address_from_witness(&self) -> Address {
        let mut credential = self.create_witness();

        credential.address = Address::from_witness_program(
            credential.witness_program.unwrap(),
            bitcoin::Network::Regtest,
        );

        add_item!(self.credentials, credential.clone());

        credential.address
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
