//! # RPC Server Starter
//!
//! This binary can start an RPC server for listening RPC calls. Can be spawned
//! multiple times. Each server will have an independent blockchain.

use bitcoin_mock_rpc::rpc::spawn_rpc_server;
use clap::Parser;
use std::process::exit;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Bitcoin Mock Rpc (C) Chainway, 2024
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Verbosity level, ranging from 0 (none) to 5 (highest)
    #[arg(short, long, default_value_t = 0)]
    verbose: u8,
    /// Optional host address
    #[arg(default_value_t = String::from("127.0.0.1"))]
    pub host: String,
    /// Optional host port (if not given, requests a random port from OS)
    #[arg(default_value_t = 0)]
    pub port: u16,
}

/// Initializes tracing.
fn initialize_logger(level: u8) {
    let level = match level {
        0 => return, // No tracing output
        1 => LevelFilter::ERROR,
        2 => LevelFilter::WARN,
        3 => LevelFilter::INFO,
        4 => LevelFilter::DEBUG,
        5 => LevelFilter::TRACE,
        _ => {
            eprintln!("Verbosity level can only be between 0 and 5 (given {level})!");
            exit(1);
        }
    };

    let layer = fmt::layer().with_test_writer();
    let filter = EnvFilter::builder()
        .with_default_directive(level.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(layer)
        .with(filter)
        .init();
}

fn main() {
    let args = Args::parse();
    initialize_logger(args.verbose);

    let server = spawn_rpc_server(Some(&args.host), Some(args.port)).unwrap();
    println!("Server started at {}", server.0);

    server.1.join().unwrap()
}
