//! # RPC Adapter Interface
//!
//! This crate provides an adapter interface that aims to mimic real Bitcoin
//! RPC interface.

mod blockchain;
mod generating;
mod rawtransactions;
mod wallet;

pub use blockchain::*;
pub use generating::*;
pub use rawtransactions::*;
pub use wallet::*;
