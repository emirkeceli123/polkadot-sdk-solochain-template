//! KOD Chain Node
//!
//! A Proof-of-Work blockchain node.
//!
//! ## Usage
//!
//! Start a mining node (auto-creates wallet):
//! ```bash
//! kod-node --mine
//! ```
//!
//! Start with specific reward address:
//! ```bash
//! kod-node --mine --reward-address YOUR_SS58_ADDRESS
//! ```
//!
//! View wallet info:
//! ```bash
//! kod-node wallet info
//! ```
//!
//! Export seed phrase:
//! ```bash
//! kod-node wallet export-seed
//! ```

#![warn(missing_docs)]

// Benchmarking module disabled - requires frame-benchmarking-cli
// which has pallet-sudo std compilation issues
// #[allow(dead_code)]
// mod benchmarking;
mod chain_spec;
mod cli;
mod command;
mod rpc;
mod service;
mod wallet;

fn main() -> sc_cli::Result<()> {
    command::run()
}
