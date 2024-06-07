//! # Bitcoin Ledger
//!
//! Mock Bitcoin ledger.

use bitcoin::TxOut;

/// Mock Bitcoin ledger structure.
pub struct Ledger {
    /// User's unspent transaction outputs.
    utxos: Vec<TxOut>,
}

impl Ledger {
	/// Creates a new empty ledger.
	pub fn new() -> Self {
		Self {
			..Default::default()
		}
	}
}

impl Default for Ledger {
	fn default() -> Self {
		Self {
			utxos: Vec::new(),
		}
	}
}
