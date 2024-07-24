//! # Spending Requirements

use super::Ledger;
use crate::ledger::errors::LedgerError;
use bitcoin::absolute;
use bitcoin::{
    ecdsa::Signature, opcodes::all::OP_PUSHBYTES_20, sighash::SighashCache, CompressedPublicKey,
    ScriptBuf, Transaction, TxOut,
};
use bitcoin::{
    key::TweakedPublicKey,
    sighash::Prevouts,
    taproot::{ControlBlock, LeafVersion},
    TapLeafHash, XOnlyPublicKey,
};
use bitcoin::{Script, TxIn, WitnessProgram};
use secp256k1::Message;

#[derive(Default)]
pub struct SpendingRequirementsReturn {
    pub taproot: Option<(TapLeafHash, Option<Vec<u8>>)>,
    pub script_buf: ScriptBuf,
    pub witness: Vec<Vec<u8>>,
}

impl Ledger {
    pub fn p2wpkh_check(
        &self,
        tx: &Transaction,
        txouts: &[TxOut],
        input_idx: usize,
    ) -> Result<(), LedgerError> {
        if txouts[input_idx].script_pubkey.len() != 22 {
            return Err(LedgerError::SpendingRequirements(
                "The ScriptPubKey is not for P2WPKH.".to_owned(),
            ));
        }

        let witness_version = txouts[input_idx].script_pubkey.as_bytes()[0];
        let witness = &tx.input[input_idx].witness;

        if witness.len() != 2 {
            return Err(LedgerError::SpendingRequirements("The number of witness elements should be exactly two (the signature and the public key).".to_owned()));
        }

        if witness_version != 0
            || txouts[input_idx].script_pubkey.as_bytes()[1] != OP_PUSHBYTES_20.to_u8()
        {
            return Err(LedgerError::SpendingRequirements(
                "The ScriptPubKey is not for P2WPKH.".to_owned(),
            ));
        }

        let pk = CompressedPublicKey::from_slice(&witness[1]).unwrap();

        let wpkh = pk.wpubkey_hash();

        if !txouts[input_idx].script_pubkey.as_bytes()[2..22].eq(AsRef::<[u8]>::as_ref(&wpkh)) {
            return Err(LedgerError::SpendingRequirements(
                "The script does not match the script public key.".to_owned(),
            ));
        }

        let sig = Signature::from_slice(&witness[0]).unwrap();

        let mut sighashcache = SighashCache::new(tx.clone());
        let h = sighashcache
            .p2wpkh_signature_hash(
                input_idx,
                &ScriptBuf::new_p2wpkh(&wpkh),
                txouts[input_idx].value,
                sig.sighash_type,
            )
            .unwrap();

        let msg = Message::from(h);
        let secp = secp256k1::Secp256k1::verification_only();
        pk.verify(&secp, &msg, &sig).unwrap();

        Ok(())
    }

    pub fn p2wsh_check(
        &self,
        tx: &Transaction,
        txouts: &[TxOut],
        input_idx: usize,
    ) -> Result<SpendingRequirementsReturn, LedgerError> {
        let witness_version = txouts[input_idx].script_pubkey.as_bytes()[0];

        if witness_version != 0 {
            return Err(LedgerError::SpendingRequirements(
                "The ScriptPubKey is not for P2WSH.".to_owned(),
            ));
        }

        let mut witness = tx.input[input_idx].witness.to_vec();

        if witness.len() < 2 {
            return Err(LedgerError::SpendingRequirements("The number of witness elements should be at least two (the empty placeholder and the script).".to_owned()));
        }

        if !witness.remove(0).is_empty() {
            return Err(LedgerError::SpendingRequirements(
                "The first witness element must be empty (aka, representing 0).".to_owned(),
            ));
        }

        let script = witness.pop().unwrap();

        let witness_program = WitnessProgram::p2wsh(&Script::from_bytes(&script));
        let sig_pub_key_expected = ScriptBuf::new_witness_program(&witness_program);

        if *txouts[input_idx].script_pubkey != sig_pub_key_expected {
            return Err(LedgerError::SpendingRequirements(
                "The script does not match the script public key.".to_owned(),
            ));
        }

        Ok(SpendingRequirementsReturn {
            taproot: None,
            script_buf: ScriptBuf::from_bytes(script.to_vec()),
            witness,
        })
    }

    pub fn p2tr_check(
        &self,
        tx: &Transaction,
        txouts: &[TxOut],
        input_idx: usize,
    ) -> Result<SpendingRequirementsReturn, LedgerError> {
        let secp = secp256k1::Secp256k1::new();

        let sig_pub_key_bytes = txouts[input_idx].script_pubkey.as_bytes();

        let witness_version = sig_pub_key_bytes[0];
        if witness_version != 0x51 {
            return Err(LedgerError::SpendingRequirements(
                "The ScriptPubKey is not for Taproot.".to_owned(),
            ));
        }

        if sig_pub_key_bytes.len() != 34 || sig_pub_key_bytes[1] != 0x20 {
            return Err(LedgerError::SpendingRequirements(
                "The ScriptPubKey does not follow the Taproot format.".to_owned(),
            ));
        }

        let mut witness = tx.input[input_idx].witness.to_vec();
        let mut annex: Option<Vec<u8>> = None;

        // Key path spend.
        if witness.len() == 1 {
            let signature = witness.pop().unwrap();
            let signature = bitcoin::taproot::Signature::from_slice(&signature).unwrap();

            let x_only_public_key = XOnlyPublicKey::from_slice(&sig_pub_key_bytes[2..]).unwrap();
            let mut sighashcache = SighashCache::new(tx.clone());
            let h = sighashcache
                .taproot_key_spend_signature_hash(
                    input_idx,
                    &Prevouts::All(&txouts),
                    signature.sighash_type,
                )
                .unwrap();

            let msg = Message::from(h);

            return match x_only_public_key.verify(&secp, &msg, &signature.signature) {
                Ok(()) => Ok(SpendingRequirementsReturn {
                    taproot: None,
                    script_buf: ScriptBuf::new(),
                    witness,
                }),
                Err(e) => Err(LedgerError::Transaction(e.to_string())),
            };
        }

        if witness.len() >= 2 && witness[witness.len() - 1][0] == 0x50 {
            annex = Some(witness.pop().unwrap());
        } else if witness.len() < 2 {
            return Err(LedgerError::SpendingRequirements("The number of witness elements should be at least two (the script and the control block).".to_owned()));
        }

        let control_block = ControlBlock::decode(&witness.pop().unwrap()).unwrap();
        let script_buf = witness.pop().unwrap();
        let script = Script::from_bytes(&script_buf);

        let out_pk = XOnlyPublicKey::from_slice(&sig_pub_key_bytes[2..]).unwrap();
        let out_pk = TweakedPublicKey::dangerous_assume_tweaked(out_pk);

        let res = control_block.verify_taproot_commitment(&secp, out_pk.to_inner(), script);
        if !res {
            return Err(LedgerError::SpendingRequirements(
                "The taproot commitment does not match the Taproot public key.".to_owned(),
            ));
        }

        Ok(SpendingRequirementsReturn {
            taproot: Some((
                TapLeafHash::from_script(script, LeafVersion::TapScript),
                annex,
            )),
            script_buf: ScriptBuf::from_bytes(script_buf),
            witness,
        })
    }

    /// Checks if given inputs are now spendable.
    pub fn check_input_lock(&self, input: TxIn) -> Result<(), LedgerError> {
        let current_block_height = self.get_block_height();
        let current_time = self.get_block_time(current_block_height)?;

        let current_block_height =
            absolute::Height::from_consensus(current_block_height as u32).unwrap();
        let current_time = absolute::Time::from_consensus(current_time as u32).unwrap();

        if let Some(tl) = self.get_utxo_timelock(input.previous_output) {
            if !tl.is_satisfied_by(current_block_height, current_time) {
                return Err(LedgerError::Script(format!("Input is locked: {:?}", input)));
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod test {
    define_pushable!();
    use crate::ledger::Ledger;
    use bitcoin::absolute::LockTime;
    use bitcoin::ecdsa::Signature;
    use bitcoin::key::UntweakedPublicKey;
    use bitcoin::secp256k1::Message;
    use bitcoin::sighash::SighashCache;
    use bitcoin::taproot::{LeafVersion, TaprootBuilder};
    use bitcoin::transaction::Version;
    use bitcoin::{
        Amount, EcdsaSighashType, OutPoint, Script, ScriptBuf, Sequence, TxIn, TxOut, Witness,
        WitnessProgram,
    };
    use bitcoin_script::{define_pushable, script};
    use bitcoin_scriptexec::utils::scriptint_vec;
    use std::str::FromStr;

    #[test]
    fn check_input_lock() {
        let ledger = Ledger::new("check_input_lock");

        let txout = ledger.create_txout(Amount::from_sat(0x45), ScriptBuf::new());
        let tx = ledger.create_transaction(vec![], vec![txout.clone()]);
        let txid = ledger.add_transaction_unconditionally(tx).unwrap();

        let txin = TxIn {
            previous_output: OutPoint { txid, vout: 0 },
            script_sig: ScriptBuf::new(),
            ..Default::default()
        };
        let tx = ledger.create_transaction(vec![txin], vec![txout]);
    }

    #[test]
    fn p2wpkh_check() {
        let ledger = Ledger::new("p2wpkh_check");
        let credential = Ledger::generate_credential_from_witness();

        let wpkh = bitcoin::PublicKey::new(credential.public_key)
            .wpubkey_hash()
            .unwrap();

        let output = TxOut {
            value: Amount::from_sat(1_000_000_000),
            script_pubkey: ScriptBuf::new_p2wpkh(&wpkh),
        };

        let tx = bitcoin::Transaction {
            version: Version::ONE,
            lock_time: LockTime::ZERO,
            input: vec![],
            output: vec![output.clone()],
        };

        let tx_id = tx.compute_txid();

        let input = TxIn {
            previous_output: OutPoint {
                txid: tx_id,
                vout: 0,
            },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::default(),
            witness: Witness::default(),
        };

        let mut tx2 = bitcoin::Transaction {
            version: Version::ONE,
            lock_time: LockTime::ZERO,
            input: vec![input.clone()],
            output: vec![],
        };

        let sighash_type = EcdsaSighashType::All;
        let mut sighashcache = SighashCache::new(tx2.clone());
        let h = sighashcache
            .p2wpkh_signature_hash(0, &output.script_pubkey, output.value, sighash_type)
            .unwrap();

        let msg = Message::from(h);
        let signature = Signature {
            signature: credential.secp.sign_ecdsa(&msg, &credential.secret_key),
            sighash_type,
        };

        tx2.input[0].witness = Witness::p2wpkh(&signature, &credential.public_key);

        ledger.p2wpkh_check(&tx2, &[output], 0).unwrap();
    }

    #[test]
    fn p2wsh_check() {
        let ledger = Ledger::new("p2wsh_check");
        let witness_program = WitnessProgram::p2wsh(Script::from_bytes(
            &script! {
                { 1234 } OP_EQUAL
            }
            .to_bytes(),
        ));

        let output = TxOut {
            value: Amount::from_sat(1_000_000_000),
            script_pubkey: ScriptBuf::new_witness_program(&witness_program),
        };

        let tx = bitcoin::Transaction {
            version: Version::ONE,
            lock_time: LockTime::ZERO,
            input: vec![],
            output: vec![output.clone()],
        };

        let tx_id = tx.compute_txid();

        let mut witness = Witness::new();
        witness.push([]);
        witness.push(scriptint_vec(1234));
        witness.push(script! { { 1234 } OP_EQUAL }.to_bytes());

        let input = TxIn {
            previous_output: OutPoint::new(tx_id, 0),
            script_sig: ScriptBuf::default(),
            sequence: Sequence::MAX,
            witness,
        };

        let tx2 = bitcoin::Transaction {
            version: Version::ONE,
            lock_time: LockTime::ZERO,
            input: vec![input.clone()],
            output: vec![],
        };

        ledger.p2wsh_check(&tx2, &[output], 0).unwrap();
    }

    #[test]
    fn p2tr_check() {
        let ledger = Ledger::new("p2tr_check");
        let secp = bitcoin::secp256k1::Secp256k1::new();
        let internal_key = UntweakedPublicKey::from(
            bitcoin::secp256k1::PublicKey::from_str(
                "0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0",
            )
            .unwrap(),
        );

        let script = script! {
            { 1234 } OP_EQUAL
        };

        let taproot_builder = TaprootBuilder::new().add_leaf(0, script.clone()).unwrap();
        let taproot_spend_info = taproot_builder.finalize(&secp, internal_key).unwrap();

        let witness_program =
            WitnessProgram::p2tr(&secp, internal_key, taproot_spend_info.merkle_root());

        let output = TxOut {
            value: Amount::from_sat(1_000_000_000),
            script_pubkey: ScriptBuf::new_witness_program(&witness_program),
        };

        let tx = bitcoin::Transaction {
            version: Version::ONE,
            lock_time: LockTime::ZERO,
            input: vec![],
            output: vec![output.clone()],
        };

        let tx_id = tx.compute_txid();

        let mut control_block_bytes = Vec::new();
        taproot_spend_info
            .control_block(&(script.clone(), LeafVersion::TapScript))
            .unwrap()
            .encode(&mut control_block_bytes)
            .unwrap();

        let mut witness = Witness::new();
        witness.push(scriptint_vec(1234));
        witness.push(script! { { 1234 } OP_EQUAL }.to_bytes());
        witness.push(control_block_bytes);

        let input = TxIn {
            previous_output: OutPoint::new(tx_id, 0),
            script_sig: ScriptBuf::default(),
            sequence: Sequence::MAX,
            witness,
        };

        let tx2 = bitcoin::Transaction {
            version: Version::ONE,
            lock_time: LockTime::ZERO,
            input: vec![input.clone()],
            output: vec![],
        };

        ledger.p2tr_check(&tx2, &[output], 0).unwrap();
    }

    #[test]
    fn p2tr_random_witness() {
        let ledger = Ledger::new("p2tr_random_witness");
        let credential = Ledger::generate_credential_from_witness();

        let output = TxOut {
            value: Amount::from_sat(1_000_000_000),
            script_pubkey: ScriptBuf::new_witness_program(&credential.witness_program.unwrap()),
        };

        let tx = bitcoin::Transaction {
            version: Version::ONE,
            lock_time: LockTime::ZERO,
            input: vec![],
            output: vec![output.clone()],
        };

        let tx_id = tx.compute_txid();

        let input = TxIn {
            previous_output: OutPoint::new(tx_id, 0),
            script_sig: ScriptBuf::default(),
            sequence: Sequence::MAX,
            witness: credential.witness.unwrap(),
        };

        let tx2 = bitcoin::Transaction {
            version: Version::ONE,
            lock_time: LockTime::ZERO,
            input: vec![input.clone()],
            output: vec![],
        };

        ledger.p2tr_check(&tx2, &[output], 0).unwrap();
    }
}
