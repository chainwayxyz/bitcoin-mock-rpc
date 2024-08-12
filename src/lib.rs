//! # Bitcoin Mock Remote Procedure Call
//!
//! This library mocks [bitcoincore-rpc](https://github.com/rust-bitcoin/rust-bitcoincore-rpc)
//! library. This mock takes the advantage of `bitcoincore-rpc` trait interface
//! called `RpcApi`.
//!
//! Applications can implement another trait that will switch between this mock
//! and the real RPC interface, for tests and production respectively.

pub mod client;
mod ledger;
pub mod rpc_adapter;

// Re-imports.
pub use client::*;
