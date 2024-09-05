//! # Bitcoin Mock Remote Procedure Call
//!
//! bitcoin-mock-rpc is an RPC library, that builds on a mock Bitcoin ledger and
//! without a wallet or consenses implementation. It aims to provide an easier
//! way to check your Bitcoin operations without you needing to setup a Bitcoin
//! environment.
//!
//! This library mocks [bitcoincore-rpc](https://github.com/rust-bitcoin/rust-bitcoincore-rpc)
//! library. This mock takes advantage of `RpcApi` trait.
//!
//! This library is built upon
//! [bitcoincore-rpc's](https://github.com/rust-bitcoin/rust-bitcoincore-rpc)
//! `RpcApi` trait interface.
//!
//! ## Interfaces
//!
//! ```text
//!           ┌────────────────────┐
//!           │                    │
//!           │  User Application  │
//!           │                    │
//!           └──▲───────────────▲─┘
//!              │               │
//!              │        ┌──────┼───────┐
//!              │        │              │
//!              │        │  RPC Server  │
//!              │        │   (Public)   │
//!              │        └──────▲───────┘
//!              │               │
//!              │        ┌──────┼────────┐
//! ┌────────────┴───┐    │               │
//! │                │    │  RPC Adapter  │
//! │  RpcApi Trait  │    │               │
//! │   Interface    ┼───►│  encodes rust │
//! │    (Public)    │    │  structs to   │
//! └───────▲────────┘    │  hex strings  │
//!         │             │               │
//!  ┌──────┴──────┐      └───────────────┘
//!  │             │
//!  │ Mock Ledger │
//!  │             │
//!  └─────────────┴
//! ```
//!
//! bitcoin-mock-rpc has 2 interfaces:
//!
//! 1. Rpc server: Similar experience as the Bitcoin RPC
//! 2. `RpcApi` trait: No servers but a direct function call to ledger
//!
//! ### RPC Server
//!
//! RPC server can be spawned as long as there are available ports for them. Each
//! server will have an independent blockchain.
//!
//! To run from CLI:
//!
//! ```bash
//! $ cargo run
//! Server started at 127.0.0.1:1024
//! #                 ^^^^^^^^^^^^^^
//! #         Use this address in applications
//! $ cargo run -- --help # Prints usage information
//! ```
//!
//! To run in a Rust application:
//!
//! ```rust
//! fn test() {
//!     // Calling `spawn_rpc_server` in a different test while this test is running
//!     // is OK and will spawn another blockchain. If parameters are the same
//!     // however, they will operate on the same blockchain. Note: (None, None)
//!     // will result to pick random values.
//!     let address = bitcoin_mock_rpc::spawn_rpc_server(None, None).unwrap();
//!
//!     let rpc =
//!         bitcoincore_rpc::Client::new(&address.0.to_string(), bitcoincore_rpc::Auth::None).unwrap();
//!
//!     // Use `bitcoincore_rpc` as is from now on. No code change is needed.
//! }
//! ```
//!
//! ### `RpcApi`
//!
//! `RpcApiWrapper` trait can be used to select between real and mock RPC. It is
//! a simple wrapper that allows you to also use methods like `Client::new()`.
//! But it needs changes in your code:
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
