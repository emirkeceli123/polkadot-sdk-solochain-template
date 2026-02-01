//! KOD Chain Node
//!
//! A Proof-of-Work blockchain node.
//!
//! ## Usage
//!
//! Start a mining node:
//! ```bash
//! kod-node --mine --reward-address YOUR_SS58_ADDRESS
//! ```
//!
//! Start a regular (non-mining) node:
//! ```bash
//! kod-node
//! ```

#![warn(missing_docs)]

#[allow(dead_code)]
mod benchmarking;
mod chain_spec;
mod cli;
mod command;
mod rpc;
mod service;

fn main() -> sc_cli::Result<()> {
    command::run()
}
