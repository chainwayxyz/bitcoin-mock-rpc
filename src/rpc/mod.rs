//! # RPC Interface
//!
//! This crate provides an RPC server that will act like the real Bitcoin RPC
//! interface.

use crate::ledger::errors::LedgerError;
use std::net::TcpListener;

/// Finds the first empty port for the given `host`.
fn find_empty_port(host: &str) -> Result<u16, LedgerError> {
    for port in 1..0xFFFFu16 {
        if let Ok(_) = TcpListener::bind((host, port)) {
            return Ok(port);
        }
    }

    Err(LedgerError::Rpc(format!(
        "No port is available for host {}",
        host
    )))
}

#[cfg(test)]
mod tests {
    #[test]
    fn find_empty_port() {
        let host = "localhost";

        println!(
            "Port {} is empty for {}",
            super::find_empty_port(host).unwrap(),
            host
        );
    }
}
