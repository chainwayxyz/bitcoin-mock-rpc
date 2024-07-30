//! # Script Related Ledger Operations

use super::{errors::LedgerError, Ledger};
use bitcoin::{
    opcodes::all::{OP_CSV, OP_PUSHNUM_1},
    relative, script, OutPoint, ScriptBuf, Sequence,
};
use bitcoin_scriptexec::{Exec, ExecCtx, Options, TxTemplate};

impl Ledger {
    pub fn run_script(
        &self,
        ctx: ExecCtx,
        tx_template: TxTemplate,
        script_buf: ScriptBuf,
        script_witness: Vec<Vec<u8>>,
    ) -> Result<(), LedgerError> {
        self.check_sequence(
            tx_template.tx.input[tx_template.input_idx].previous_output,
            script_buf.clone(),
            tx_template.tx.input[tx_template.input_idx].sequence.0,
        )?;

        let mut exec = Exec::new(
            ctx,
            Options::default(),
            tx_template,
            script_buf.clone(),
            script_witness,
        )
        .map_err(|e| LedgerError::SpendingRequirements(format!("Script format error: {:?}", e)))?;

        loop {
            let res = exec.exec_next();
            if res.is_err() {
                break;
            }
        }

        let res = exec.result().unwrap();
        if !res.success {
            return Err(LedgerError::Script(format!(
                "The script execution is not successful: {:?}",
                res
            )));
        }

        Ok(())
    }

    #[inline]
    pub fn sequence_to_timelock(sequence: u32) -> Result<relative::LockTime, LedgerError> {
        match relative::LockTime::from_sequence(Sequence::from_consensus(sequence)) {
            Ok(lt) => return Ok(lt),
            Err(e) => {
                return Err(LedgerError::Script(format!(
                    "Couldn't convert sequence {} to timelock: {}",
                    sequence, e
                )))
            }
        };
    }

    /// Checks if a script is a CSV script. If it is, returns lock time.
    fn is_csv(script_buf: ScriptBuf) -> Option<u32> {
        let mut instructions = script_buf.instructions();
        let op1 = instructions.next();
        let op2 = instructions.next();

        if let (Some(Ok(op1)), Some(Ok(op2))) = (op1, op2) {
            if op2 == script::Instruction::Op(OP_CSV) {
                let op1_data: i64;

                if let Some(bytes) = op1.push_bytes() {
                    op1_data =
                        bitcoin_scriptexec::utils::read_scriptint_size(bytes.as_bytes(), 5, true)
                            .unwrap();
                } else {
                    let data = op1.opcode().unwrap().to_u8();
                    let data = data - (OP_PUSHNUM_1.to_u8() - 1);
                    op1_data = data as i64;
                };

                return Some(op1_data as u32);
            }
        }

        None
    }

    /// Checks if it is a CSV script and compares sequence against the current
    /// block height/time.
    fn check_sequence(
        &self,
        utxo: OutPoint,
        script_buf: ScriptBuf,
        input_sequence: u32,
    ) -> Result<(), LedgerError> {
        // If not a CSV script, we don't need to check sequence.
        match Ledger::is_csv(script_buf) {
            Some(_) => (),
            None => return Ok(()),
        };

        let current_block_height = self.get_block_height();
        let current_block_time = self.get_block_time(current_block_height)?;

        let tx_block_height = self.get_transaction_block_height(&utxo.txid)?;
        let tx_block_time = self.get_tx_block_height(utxo.txid);

        let blocks_after = current_block_height - tx_block_height;
        let time_after = current_block_time - tx_block_time;

        let sequence_lock = Ledger::sequence_to_timelock(input_sequence)?;

        match sequence_lock {
            relative::LockTime::Blocks(height) => {
                if height.value() > blocks_after as u16 {
                    return Err(LedgerError::Script(format!(
                        "Input {:?} is locked until block {} (current block height {})",
                        utxo,
                        tx_block_height + height.value() as u32,
                        current_block_height,
                    )));
                }
            }
            relative::LockTime::Time(time) => {
                if time.value() > time_after as u16 {
                    return Err(LedgerError::Script(format!(
                        "Input {:?} is locked until time {} (current block time {})",
                        utxo,
                        tx_block_time + time.value() as u32,
                        current_block_time
                    )));
                }
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ledger::{self, Ledger};
    use bitcoin::opcodes::all::*;
    use bitcoin::script::Builder;
    use bitcoin::{Amount, OutPoint, Sequence};

    #[test]
    fn check_for_csv_with_block_height() {
        let ledger = Ledger::new("check_for_csv_with_block_height");
        let credential = ledger::Ledger::generate_credential_from_witness();
        let xonly_pk = credential.x_only_public_key;

        let txout = ledger.create_txout(Amount::from_sat(0x45), credential.address.script_pubkey());
        let tx = ledger.create_transaction(vec![], vec![txout]);
        let utxo = OutPoint {
            txid: tx.compute_txid(),
            vout: 0,
        };

        ledger.add_transaction_unconditionally(tx.clone()).unwrap();
        ledger.increment_block_height();
        ledger.increment_block_height();
        assert_eq!(ledger.get_block_height(), 2);

        let script = Builder::new()
            .push_int(0x1 as i64)
            .push_opcode(OP_CSV)
            .push_opcode(OP_DROP)
            .push_x_only_key(&xonly_pk)
            .push_opcode(OP_CHECKSIG)
            .into_script();
        ledger.check_sequence(utxo, script, 2).unwrap();

        for _ in 0..3 {
            ledger.increment_block_height();
        }
        assert_eq!(ledger.get_block_height(), 5);

        let script = Builder::new()
            .push_int(0x1 as i64)
            .push_opcode(OP_CSV)
            .push_opcode(OP_DROP)
            .push_x_only_key(&xonly_pk)
            .push_opcode(OP_CHECKSIG)
            .into_script();
        ledger.check_sequence(utxo, script, 1).unwrap();

        for _ in 0..3 {
            ledger.increment_block_height();
        }
        assert_eq!(ledger.get_block_height(), 8);
        let script = Builder::new()
            .push_int(0x45 as i64)
            .push_opcode(OP_CSV)
            .push_opcode(OP_DROP)
            .push_x_only_key(&xonly_pk)
            .push_opcode(OP_CHECKSIG)
            .into_script();
        if let Ok(_) = ledger.check_sequence(utxo, script, 0x45) {
            assert!(false);
        }

        for _ in 0..0x100 {
            ledger.increment_block_height();
        }
        assert_eq!(ledger.get_block_height(), 8 + 0x100);
        let script = Builder::new()
            .push_int(0x100 as i64)
            .push_opcode(OP_CSV)
            .push_opcode(OP_DROP)
            .push_x_only_key(&xonly_pk)
            .push_opcode(OP_CHECKSIG)
            .into_script();
        ledger.check_sequence(utxo, script, 0x100).unwrap();
    }

    #[test]
    #[ignore = "Needs more work with creating inputs"]
    fn check_csv_with_time_lock() {
        let ledger = Ledger::new("check_csv_with_time_lock");
        let credential = ledger::Ledger::generate_credential_from_witness();
        let xonly_pk = credential.x_only_public_key;

        let txout = ledger.create_txout(Amount::from_sat(0x45), credential.address.script_pubkey());
        let tx = ledger.create_transaction(vec![], vec![txout]);
        let utxo = OutPoint {
            txid: tx.compute_txid(),
            vout: 0,
        };

        ledger.add_transaction_unconditionally(tx.clone()).unwrap();
        ledger.increment_block_height();

        let sequence = Sequence::from_512_second_intervals(2);
        let script = Builder::new()
            .push_sequence(sequence)
            .push_opcode(OP_CSV)
            .push_opcode(OP_DROP)
            .push_x_only_key(&xonly_pk)
            .push_opcode(OP_CHECKSIG)
            .into_script();
        ledger
            .check_sequence(utxo, script, sequence.to_consensus_u32())
            .unwrap();

        ledger.increment_block_height();

        let sequence = Sequence::from_512_second_intervals(0x45);
        let script = Builder::new()
            .push_sequence(sequence)
            .push_opcode(OP_CSV)
            .push_opcode(OP_DROP)
            .push_x_only_key(&xonly_pk)
            .push_opcode(OP_CHECKSIG)
            .into_script();
        if let Ok(_) = ledger.check_sequence(
            utxo,
            script,
            Sequence::from_512_second_intervals(0x44).to_consensus_u32(),
        ) {
            assert!(false);
        };

        ledger.increment_block_height();

        let sequence = Sequence::from_512_second_intervals(300);
        let script = Builder::new()
            .push_sequence(sequence)
            .push_opcode(OP_CSV)
            .push_opcode(OP_DROP)
            .push_x_only_key(&xonly_pk)
            .push_opcode(OP_CHECKSIG)
            .into_script();
        ledger
            .check_sequence(utxo, script, sequence.to_consensus_u32())
            .unwrap();
    }
}
