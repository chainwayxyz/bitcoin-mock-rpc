//! # Script Related Ledger Operations

use super::{errors::LedgerError, Ledger};
use bitcoin::ScriptBuf;
use bitcoin_scriptexec::{Exec, ExecCtx, Options, TxTemplate};

impl Ledger {
    pub fn run_script(
        ctx: ExecCtx,
        tx_template: TxTemplate,
        script_buf: ScriptBuf,
        script_witness: Vec<Vec<u8>>,
    ) -> Result<(), LedgerError> {
        let mut exec = Exec::new(
            ctx,
            Options::default(),
            tx_template,
            script_buf,
            script_witness,
        )
        .map_err(|e| LedgerError::SpendingRequirements(format!("Script format error: {:?}", e)))?;

        loop {
            println!("res adim {:?}", exec.remaining_script());
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
