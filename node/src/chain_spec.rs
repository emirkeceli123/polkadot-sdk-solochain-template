//! Chain specification for KOD Chain.

use sc_service::ChainType;
use kod_runtime::WASM_BINARY;

/// Specialized `ChainSpec` for KOD Chain.
pub type ChainSpec = sc_service::GenericChainSpec;

/// Development chain specification (single node, instant mining)
pub fn development_chain_spec() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
        None,
    )
    .with_name("KOD Development")
    .with_id("kod_dev")
    .with_chain_type(ChainType::Development)
    .with_genesis_config_preset_name(sp_genesis_builder::DEV_RUNTIME_PRESET)
    .build())
}

/// Local testnet chain specification (multiple nodes)
pub fn local_chain_spec() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
        None,
    )
    .with_name("KOD Local Testnet")
    .with_id("kod_local")
    .with_chain_type(ChainType::Local)
    .with_genesis_config_preset_name(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET)
    .build())
}

/// Live mainnet chain specification
pub fn mainnet_chain_spec() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "Mainnet wasm not available".to_string())?,
        None,
    )
    .with_name("KOD Chain")
    .with_id("kod_mainnet")
    .with_chain_type(ChainType::Live)
    .with_genesis_config_preset_name(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET)
    .build())
}
