//! # Script Related Ledger Operations

use super::{errors::LedgerError, Ledger};
use bitcoin::{
    opcodes::all::{OP_CSV, OP_PUSHNUM_1},
    relative::{Height, LockTime},
    script, ScriptBuf, Sequence,
};
use bitcoin_scriptexec::{Exec, ExecCtx, Options, TxTemplate};

impl Ledger {
    pub fn run_script(
        &self,
        ctx: ExecCtx,
        tx_template: TxTemplate,
        script_buf: ScriptBuf,
        script_witness: Vec<Vec<u8>>,
        input_block_heights: &Vec<u64>,
    ) -> Result<(), LedgerError> {
        let _prev_outs = tx_template.prevouts.clone();
        let input_idx = tx_template.input_idx.clone();

        let mut exec = Exec::new(
            ctx,
            Options::default(),
            tx_template,
            script_buf.clone(),
            script_witness,
        )
        .map_err(|e| LedgerError::SpendingRequirements(format!("Script format error: {:?}", e)))?;

        self.check_csv(script_buf, input_block_heights, input_idx)?;

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
    fn check_csv(
        &self,
        script_buf: ScriptBuf,
        _input_block_heights: &Vec<u64>,
        _input_idx: usize,
    ) -> Result<(), LedgerError> {
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
                    let height = op1.opcode().unwrap().to_u8();
                    let height = height - (OP_PUSHNUM_1.to_u8() - 1);
                    op1_data = height as i64;
                };

                let lock_time =
                    match LockTime::from_sequence(Sequence::from_consensus(op1_data as u32)) {
                        Ok(lt) => lt,
                        Err(e) => return Err(LedgerError::Script(e.to_string())),
                    };

                if lock_time.is_block_height() {
                    let current_height = self.get_block_height();
                    let target_height = Height::from_height(current_height as u16);

                    if let false = lock_time.is_satisfied_by_height(target_height).unwrap() {
                        return Err(LedgerError::Script(format!(
                            "UTXO is locked for the block height: {op1_data}; Current block height: {current_height}"
                        )));
                    }
                } else {
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ledger::{self, Ledger};
    use bitcoin::opcodes::all::*;
    use bitcoin::script::Builder;

    #[test]
    fn check_csv_with_block_height() {
        let ledger = Ledger::new("check_csv");
        let xonly_pk = ledger::Ledger::generate_credential_from_witness().x_only_public_key;
        let mut input_block_heights: Vec<u64> = Vec::new();

        let script = Builder::new()
            .push_int(0x1 as i64)
            .push_opcode(OP_CSV)
            .push_opcode(OP_DROP)
            .push_x_only_key(&xonly_pk)
            .push_opcode(OP_CHECKSIG)
            .into_script();
        if let Ok(_) = ledger.check_csv(script, &input_block_heights, 0) {
            assert!(false);
        };

        for i in 0..2 {
            ledger.increment_block_height();
            input_block_heights.push(i);
        }

        let script = Builder::new()
            .push_int(0x1 as i64)
            .push_opcode(OP_CSV)
            .push_opcode(OP_DROP)
            .push_x_only_key(&xonly_pk)
            .push_opcode(OP_CHECKSIG)
            .into_script();
        ledger.check_csv(script, &input_block_heights, 0).unwrap();

        for i in 2..0x46 {
            ledger.increment_block_height();
            input_block_heights.push(i);
        }

        let script = Builder::new()
            .push_int(0x45 as i64)
            .push_opcode(OP_CSV)
            .push_opcode(OP_DROP)
            .push_x_only_key(&xonly_pk)
            .push_opcode(OP_CHECKSIG)
            .into_script();
        ledger.check_csv(script, &input_block_heights, 0).unwrap();

        let script = Builder::new()
            .push_int(0x100 as i64)
            .push_opcode(OP_CSV)
            .push_opcode(OP_DROP)
            .push_x_only_key(&xonly_pk)
            .push_opcode(OP_CHECKSIG)
            .into_script();
        if let Ok(_) = ledger.check_csv(script, &input_block_heights, 0) {
            assert!(false);
        };
    }
}
