//! # Bitcoin Mock Remote Procedure Call
//!
//! bitcoin-mock-rpc is an RPC library, that builds on a mock Bitcoin ledger and
//! without a wallet or consenses implementation. It aims to provide an easier
//! way to check your Bitcoin operations without you needing to setup a Bitcoin
//! environment.
//!
//! This library mocks
//! [bitcoincore-rpc](https://github.com/rust-bitcoin/rust-bitcoincore-rpc)
//! library. This mock takes advantage of `RpcApi` trait.
//!
//! ## Interface
//!
//! ```text
//! ┌────────────────────┐
//! │                    │
//! │  User Application  │
//! │                    │
//! └────────▲───────────┘
//!          │
//!  ┌───────┴────────┐
//!  │                │
//!  │  RpcApi Trait  │
//!  │   Interface    │
//!  │                │
//!  └───────▲────────┘
//!          │
//!  ┌───────┴─────┐
//!  │             │
//!  │ Mock Ledger │
//!  │             │
//!  └─────────────┴
//! ```
//!
//! `RpcApiWrapper` trait can be used to select between real and mock RPC. It is
//! a simple wrapper that allows you to also use methods like `Client::new()`.
//! Needs changes in your code:
//!
//! ```rust
//! use bitcoin_mock_rpc::RpcApiWrapper;
//!
//! struct MyStruct<R: RpcApiWrapper> {
//!     data: u32,
//!     rpc: R,
//! }
//!
//! fn my_func() {
//!     let strct = MyStruct {
//!         data: 0x45,
//!         // This will connect to Bitcoin RPC.
//!         rpc: bitcoincore_rpc::Client::new("127.0.0.1", bitcoincore_rpc::Auth::None).unwrap(),
//!     };
//!
//!     // Do stuff...
//! }
//!
//! fn test() {
//!     let strct = MyStruct {
//!         data: 0x1F,
//!         // This will connect to mock RPC.
//!         rpc: bitcoin_mock_rpc::Client::new("db_name", bitcoincore_rpc::Auth::None).unwrap(),
//!     };
//!
//!     // Do stuff...
//! }
//! ```

pub mod client;
mod ledger;
mod utils;

// Re-imports.
pub use client::*;

#[cfg(feature = "rpc_server")]
pub mod rpc;
#[cfg(feature = "rpc_server")]
pub use rpc::*;
