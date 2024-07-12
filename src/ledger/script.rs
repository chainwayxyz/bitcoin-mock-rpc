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
    ) -> Result<(), LedgerError> {
        let _prev_outs = tx_template.prevouts.clone();
        let _input_idx = tx_template.input_idx.clone();

        let mut exec = Exec::new(
            ctx,
            Options::default(),
            tx_template,
            script_buf.clone(),
            script_witness,
        )
        .map_err(|e| LedgerError::SpendingRequirements(format!("Script format error: {:?}", e)))?;

        // Check for CSV
        if {
            let mut instructions = script_buf.instructions();
            let op1 = instructions.next();
            let op2 = instructions.next();

            if let (Some(Ok(op1)), Some(Ok(op2))) = (op1, op2) {
                if op2 == script::Instruction::Op(OP_CSV) {
                    let height = op1.opcode().unwrap().to_u8();
                    let height = height - (OP_PUSHNUM_1.to_u8() - 1);

                    let current_height = self.get_block_height();

                    println!(
                        "-- {:?}, {:?}, {:?}, {:?}, ",
                        op1, op2, height, current_height
                    );
                }
            }

            true
        } == false
        {
            return Err(LedgerError::Script(format!(
                "TX is locked for the block height: "
            )));
        }

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
}
