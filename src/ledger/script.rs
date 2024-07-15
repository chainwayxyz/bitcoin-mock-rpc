//! # Script Related Ledger Operations

use super::{errors::LedgerError, Ledger};
use bitcoin::{
    opcodes::all::{OP_CSV, OP_PUSHNUM_1},
    script, ScriptBuf,
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
        input_block_heights: &Vec<u64>,
        input_idx: usize,
    ) -> Result<(), LedgerError> {
        let mut instructions = script_buf.instructions();
        let op1 = instructions.next();
        let op2 = instructions.next();

        if let (Some(Ok(op1)), Some(Ok(op2))) = (op1, op2) {
            if op2 == script::Instruction::Op(OP_CSV) {
                let height = op1.opcode().unwrap().to_u8();
                let height = height - (OP_PUSHNUM_1.to_u8() - 1);

                let current_height = self.get_block_height();

                if current_height - input_block_heights[input_idx] < height as u64 {
                    return Err(LedgerError::Script(format!(
                        "UTXO is locked for the block height: {height}; Current block height: {current_height}"
                    )));
                }
            }
        }

        Ok(())
    }
}
