//! # Config
//!
//! This crate mocks a configuration interface. This interface can be used for
//! testing.

use bitcoin::Network;
use serde::{Deserialize, Serialize};

/// Mock configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Bitcoin network to work on.
    pub network: Network,
    /// Number of verifiers.
    pub num_verifiers: usize,
    /// Minimum relay fee.
    pub min_relay_fee: u64,
    /// User takes after.
    pub user_takes_after: u32,
    /// Threshold for confirmation.
    pub confirmation_treshold: u32,
}

impl Config {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            num_verifiers: 4,
            min_relay_fee: 289,
            user_takes_after: 200,
            confirmation_treshold: 1,
            network: Network::Regtest,
        }
    }
}
