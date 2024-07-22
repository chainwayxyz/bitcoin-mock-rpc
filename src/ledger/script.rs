//! # Script Related Ledger Operations

use super::{errors::LedgerError, Ledger};
use bitcoin::{
    opcodes::all::{OP_CSV, OP_PUSHNUM_1},
    relative::{self, Height, Time},
    script, OutPoint, ScriptBuf, Sequence,
};
use bitcoin_scriptexec::{Exec, ExecCtx, Options, TxTemplate};

impl Ledger {
    pub fn run_script(
        &self,
        ctx: ExecCtx,
        tx_template: TxTemplate,
        script_buf: ScriptBuf,
        script_witness: Vec<Vec<u8>>,
        prevouts: &[OutPoint],
    ) -> Result<(), LedgerError> {
        let prev_outs = tx_template.prevouts.clone();

        // Chech if inputs satisfies their locks.
        for prevout in prevouts {
            self.check_csv(prevout.clone(), prev_outs[0].script_pubkey.clone())?;
        }

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

    /// Checks if script is a CSV and it satisfies conditions.
    fn check_csv(&self, utxo: OutPoint, script_buf: ScriptBuf) -> Result<(), LedgerError> {
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

                let lock_time = match relative::LockTime::from_sequence(Sequence::from_consensus(
                    op1_data as u32,
                )) {
                    Ok(lt) => lt,
                    Err(e) => return Err(LedgerError::Script(e.to_string())),
                };

                // When this UTXO added to the blockchain?
                // TODO: Not exactly block_time. Maybe change naming?
                // let block_time = self.get_utxo_timelock()

                if lock_time.is_block_height() {
                    let current_height = self.get_block_height();
                    let target_height = Height::from_height(current_height as u16);

                    if let false = lock_time.is_satisfied_by_height(target_height).unwrap() {
                        return Err(LedgerError::Script(format!(
                            "UTXO is locked for the block height: {target_height}; Current block height: {current_height}"
                        )));
                    }
                } else {
                    let current_height = self.get_block_height();
                    let current_time = self.get_block_time(current_height).unwrap();

                    let target_time = Time::from_seconds_floor(current_time as u32).unwrap();

                    if let false = lock_time.is_satisfied_by_time(target_time).unwrap() {
                        return Err(LedgerError::Script(format!(
                            "UTXO is locked for the block time: {target_time}; Current block time: {current_time}"
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ledger::{self, Ledger};
    use bitcoin::hashes::Hash;
    use bitcoin::opcodes::all::*;
    use bitcoin::script::Builder;
    use bitcoin::{OutPoint, Sequence, Txid};

    #[test]
    fn check_csv_with_block_height() {
        let ledger = Ledger::new("check_csv_with_block_height");
        let xonly_pk = ledger::Ledger::generate_credential_from_witness().x_only_public_key;

        let script = Builder::new()
            .push_int(0x1 as i64)
            .push_opcode(OP_CSV)
            .push_opcode(OP_DROP)
            .push_x_only_key(&xonly_pk)
            .push_opcode(OP_CHECKSIG)
            .into_script();
        if let Ok(_) = ledger.check_csv(
            OutPoint {
                txid: Txid::all_zeros(),
                vout: 0,
            },
            script,
        ) {
            assert!(false);
        };

        for _ in 0..2 {
            ledger.increment_block_height();
        }
        let script = Builder::new()
            .push_int(0x1 as i64)
            .push_opcode(OP_CSV)
            .push_opcode(OP_DROP)
            .push_x_only_key(&xonly_pk)
            .push_opcode(OP_CHECKSIG)
            .into_script();
        ledger
            .check_csv(
                OutPoint {
                    txid: Txid::all_zeros(),
                    vout: 0,
                },
                script,
            )
            .unwrap();

        for _ in 2..0x46 {
            ledger.increment_block_height();
        }
        let script = Builder::new()
            .push_int(0x45 as i64)
            .push_opcode(OP_CSV)
            .push_opcode(OP_DROP)
            .push_x_only_key(&xonly_pk)
            .push_opcode(OP_CHECKSIG)
            .into_script();
        ledger
            .check_csv(
                OutPoint {
                    txid: Txid::all_zeros(),
                    vout: 0,
                },
                script,
            )
            .unwrap();

        let script = Builder::new()
            .push_int(0x100 as i64)
            .push_opcode(OP_CSV)
            .push_opcode(OP_DROP)
            .push_x_only_key(&xonly_pk)
            .push_opcode(OP_CHECKSIG)
            .into_script();
        if let Ok(_) = ledger.check_csv(
            OutPoint {
                txid: Txid::all_zeros(),
                vout: 0,
            },
            script,
        ) {
            assert!(false);
        };
    }

    #[ignore]
    #[test]
    fn check_csv_with_time_lock() {
        let ledger = Ledger::new("check_csv_with_time_lock");
        let xonly_pk = ledger::Ledger::generate_credential_from_witness().x_only_public_key;

        let sequence = Sequence::from_512_second_intervals(2);
        let script = Builder::new()
            .push_sequence(sequence)
            .push_opcode(OP_CSV)
            .push_opcode(OP_DROP)
            .push_x_only_key(&xonly_pk)
            .push_opcode(OP_CHECKSIG)
            .into_script();
        println!("Script: {}", script);
        if let Ok(_) = ledger.check_csv(
            OutPoint {
                txid: Txid::all_zeros(),
                vout: 0,
            },
            script,
        ) {
            assert!(false);
        };

        for _ in 0..3 {
            ledger.increment_block_height();
        }
        let sequence = Sequence::from_512_second_intervals(3);
        let script = Builder::new()
            .push_sequence(sequence)
            .push_opcode(OP_CSV)
            .push_opcode(OP_DROP)
            .push_x_only_key(&xonly_pk)
            .push_opcode(OP_CHECKSIG)
            .into_script();
        println!("Script: {}", script);
        ledger
            .check_csv(
                OutPoint {
                    txid: Txid::all_zeros(),
                    vout: 0,
                },
                script,
            )
            .unwrap();
    }
}
