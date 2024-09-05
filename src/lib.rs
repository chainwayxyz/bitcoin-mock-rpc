//! # Bitcoin Mock Remote Procedure Call
//!
//! This library mocks [bitcoincore-rpc](https://github.com/rust-bitcoin/rust-bitcoincore-rpc)
//! library. This mock takes advantage of `RpcApi` trait.
//!
//! Applications can implement another trait that will switch between this mock
//! and the real RPC interface, for tests and production respectively.

pub mod client;
mod ledger;
mod utils;

// Re-imports.
pub use client::*;

#[cfg(feature = "rpc_server")]
pub mod rpc;
