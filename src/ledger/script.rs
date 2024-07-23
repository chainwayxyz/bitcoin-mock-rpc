//! # Script Related Ledger Operations

use super::{errors::LedgerError, Ledger};
use bitcoin::{
    absolute::{self, Height, Time},
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
        self.check_for_csv(
            tx_template.tx.input[tx_template.input_idx].previous_output,
            script_buf.clone(),
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
            Err(e) => return Err(LedgerError::AnyHow(e.into())),
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

    /// Checks if script is a CSV and saves lock time to ledger if it is.
    fn check_for_csv(&self, utxo: OutPoint, script_buf: ScriptBuf) -> Result<(), LedgerError> {
        let sequence = match Ledger::is_csv(script_buf) {
            Some(s) => s,
            None => return Ok(()),
        };
        let lock_time = Ledger::sequence_to_timelock(sequence)?;

        match lock_time {
            relative::LockTime::Blocks(height) => {
                let target_height = self.get_block_height() as u32 + height.to_consensus_u32();
                let lock =
                    absolute::LockTime::Blocks(Height::from_consensus(target_height).unwrap());

                self.add_utxo_with_lock_time(utxo, lock);
            }
            relative::LockTime::Time(time) => {
                let target_time = self.get_block_time(self.get_block_height()).unwrap() as u32
                    + time.to_consensus_u32();
                let lock = absolute::LockTime::Seconds(Time::from_consensus(target_time).unwrap());

                self.add_utxo_with_lock_time(utxo, lock);
            }
        };

        // if lock_time.is_block_height() {
        //     let current_height = self.get_block_height();
        //     let target_height = Height::from_height(current_height as u16);

        //     if let false = lock_time.is_satisfied_by_height(target_height).unwrap() {
        //         return Err(LedgerError::Script(format!(
        //             "UTXO is locked for the block height: {target_height}; Current block height: {current_height}"
        //         )));
        //     }
        // } else {
        //     let current_height = self.get_block_height();
        //     let current_time = self.get_block_time(current_height).unwrap();

        //     let target_time = Time::from_seconds_floor(current_time as u32).unwrap();

        //     if let false = lock_time.is_satisfied_by_time(target_time).unwrap() {
        //         return Err(LedgerError::Script(format!(
        //             "UTXO is locked for the block time: {target_time}; Current block time: {current_time}"
        //         )));
        //     }
        // }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use crate::ledger::{self, Ledger};
    // use bitcoin::opcodes::all::*;
    // use bitcoin::script::Builder;
    // use bitcoin::Sequence;

    #[test]
    fn check_csv_with_block_height() {
        //     let ledger = Ledger::new("check_csv_with_block_height");
        //     let xonly_pk = ledger::Ledger::generate_credential_from_witness().x_only_public_key;

        //     let script = Builder::new()
        //         .push_int(0x1 as i64)
        //         .push_opcode(OP_CSV)
        //         .push_opcode(OP_DROP)
        //         .push_x_only_key(&xonly_pk)
        //         .push_opcode(OP_CHECKSIG)
        //         .into_script();
        //     if let Ok(_) = ledger.check_for_csv(bitcoin::OutPoint { txid: (), vout: () }, script) {
        //         assert!(false);
        //     };

        //     for _ in 0..2 {
        //         ledger.increment_block_height();
        //     }
        //     let script = Builder::new()
        //         .push_int(0x1 as i64)
        //         .push_opcode(OP_CSV)
        //         .push_opcode(OP_DROP)
        //         .push_x_only_key(&xonly_pk)
        //         .push_opcode(OP_CHECKSIG)
        //         .into_script();
        //     ledger.check_for_csv(script).unwrap();

        //     for _ in 2..0x46 {
        //         ledger.increment_block_height();
        //     }
        //     let script = Builder::new()
        //         .push_int(0x45 as i64)
        //         .push_opcode(OP_CSV)
        //         .push_opcode(OP_DROP)
        //         .push_x_only_key(&xonly_pk)
        //         .push_opcode(OP_CHECKSIG)
        //         .into_script();
        //     ledger.check_for_csv(script).unwrap();

        //     let script = Builder::new()
        //         .push_int(0x100 as i64)
        //         .push_opcode(OP_CSV)
        //         .push_opcode(OP_DROP)
        //         .push_x_only_key(&xonly_pk)
        //         .push_opcode(OP_CHECKSIG)
        //         .into_script();
        //     if let Ok(_) = ledger.check_for_csv(script) {
        //         assert!(false);
        //     };
    }

    #[ignore]
    #[test]
    fn check_csv_with_time_lock() {
        // let ledger = Ledger::new("check_csv_with_time_lock");
        // let xonly_pk = ledger::Ledger::generate_credential_from_witness().x_only_public_key;

        // let sequence = Sequence::from_512_second_intervals(2);
        // let script = Builder::new()
        //     .push_sequence(sequence)
        //     .push_opcode(OP_CSV)
        //     .push_opcode(OP_DROP)
        //     .push_x_only_key(&xonly_pk)
        //     .push_opcode(OP_CHECKSIG)
        //     .into_script();
        // println!("Script: {}", script);
        // if let Ok(_) = ledger.check_for_csv(script) {
        //     assert!(false);
        // };

        // for _ in 0..3 {
        //     ledger.increment_block_height();
        // }
        // let sequence = Sequence::from_512_second_intervals(3);
        // let script = Builder::new()
        //     .push_sequence(sequence)
        //     .push_opcode(OP_CSV)
        //     .push_opcode(OP_DROP)
        //     .push_x_only_key(&xonly_pk)
        //     .push_opcode(OP_CHECKSIG)
        //     .into_script();
        // println!("Script: {}", script);
        // ledger.check_for_csv(script).unwrap();
    }
}
