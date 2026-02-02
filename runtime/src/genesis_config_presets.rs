//! Genesis configuration presets for KOD Chain.
//!
//! This module defines the initial state of the blockchain:
//! - Total supply: 1,000,000,000 KOD (1 Billion)
//! - Mining reserve: 900,000,000 KOD (for ~8 years of block rewards)
//! - Foundation/Team: 100,000,000 KOD (for development, marketing, liquidity)

use crate::{AccountId, BalancesConfig, RuntimeGenesisConfig, SudoConfig};
use alloc::{vec, vec::Vec};
use frame_support::{build_struct_json_patch, PalletId};
use serde_json::Value;
use sp_genesis_builder::{self, PresetId};
use sp_keyring::Sr25519Keyring;
use sp_runtime::traits::AccountIdConversion;

use crate::{FOUNDATION_RESERVE, MINING_RESERVE, TOTAL_SUPPLY, UNIT};

/// Get the mining reserve account address
fn mining_reserve_account() -> AccountId {
    PalletId(*b"mineresv").into_account_truncating()
}

// Compile-time assertion: Mining reserve + Foundation = Total supply
const _: () = assert!(
    MINING_RESERVE as u128 + FOUNDATION_RESERVE as u128 == TOTAL_SUPPLY as u128,
    "Genesis allocations must equal TOTAL_SUPPLY (1,000,000,000 KOD)"
);

/// Create genesis config with the given endowed accounts
fn kod_genesis(
    endowed_accounts: Vec<(AccountId, u128)>,
    root: AccountId,
) -> Value {
    // Calculate total endowed (should be FOUNDATION_RESERVE)
    let total_endowed: u128 = endowed_accounts.iter().map(|(_, b)| b).sum();
    
    // Mining reserve gets its allocation
    let mining_reserve_balance = MINING_RESERVE;
    
    // Verify total
    assert!(
        total_endowed + mining_reserve_balance <= TOTAL_SUPPLY,
        "Total allocations exceed TOTAL_SUPPLY"
    );
    
    // Combine all balances
    let mut all_balances = endowed_accounts;
    all_balances.push((mining_reserve_account(), mining_reserve_balance));
    
    build_struct_json_patch!(RuntimeGenesisConfig {
        balances: BalancesConfig {
            balances: all_balances,
        },
        sudo: SudoConfig { key: Some(root) },
    })
}

/// Development genesis config - for local testing
/// Gives more coins to test accounts for easier testing
pub fn development_config_genesis() -> Value {
    // For development, distribute 100M KOD among test accounts
    // These are from the FOUNDATION_RESERVE
    let endowed = vec![
        (Sr25519Keyring::Alice.to_account_id(), 50_000_000 * UNIT),   // 50M KOD
        (Sr25519Keyring::Bob.to_account_id(), 25_000_000 * UNIT),     // 25M KOD
        (Sr25519Keyring::Charlie.to_account_id(), 12_500_000 * UNIT), // 12.5M KOD
        (Sr25519Keyring::Dave.to_account_id(), 6_250_000 * UNIT),     // 6.25M KOD
        (Sr25519Keyring::Eve.to_account_id(), 6_250_000 * UNIT),      // 6.25M KOD
    ];
    
    // Verify allocation sums to FOUNDATION_RESERVE
    let total: u128 = endowed.iter().map(|(_, b)| b).sum();
    assert_eq!(total, FOUNDATION_RESERVE, "Foundation allocation must equal 100,000,000 KOD");
    
    kod_genesis(
        endowed,
        Sr25519Keyring::Alice.to_account_id(),
    )
}

/// Local testnet genesis config
/// Same as development but can be customized for multi-node testing
pub fn local_config_genesis() -> Value {
    // Distribute 100M KOD among test accounts
    let endowed = vec![
        (Sr25519Keyring::Alice.to_account_id(), 50_000_000 * UNIT),   // 50M KOD
        (Sr25519Keyring::Bob.to_account_id(), 25_000_000 * UNIT),     // 25M KOD
        (Sr25519Keyring::Charlie.to_account_id(), 12_500_000 * UNIT), // 12.5M KOD
        (Sr25519Keyring::Dave.to_account_id(), 6_250_000 * UNIT),     // 6.25M KOD
        (Sr25519Keyring::Eve.to_account_id(), 6_250_000 * UNIT),      // 6.25M KOD
    ];
    
    kod_genesis(
        endowed,
        Sr25519Keyring::Alice.to_account_id(),
    )
}

/// Provides the JSON representation of predefined genesis config for given `id`.
pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
    let patch = match id.as_ref() {
        sp_genesis_builder::DEV_RUNTIME_PRESET => development_config_genesis(),
        sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET => local_config_genesis(),
        _ => return None,
    };
    Some(
        serde_json::to_string(&patch)
            .expect("serialization to json is expected to work. qed.")
            .into_bytes(),
    )
}

/// List of supported presets.
pub fn preset_names() -> Vec<PresetId> {
    vec![
        PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET),
        PresetId::from(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET),
    ]
}
