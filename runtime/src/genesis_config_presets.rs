//! Genesis configuration presets for KOD Chain.
//!
//! This module defines the initial state of the blockchain:
//! - Total supply: 4,000,000 KOD
//! - Mining reserve: 3,000,000 KOD (for block rewards)
//! - Team/community: 1,000,000 KOD

use crate::{AccountId, BalancesConfig, RuntimeGenesisConfig, SudoConfig};
use alloc::{vec, vec::Vec};
use frame_support::{build_struct_json_patch, traits::Get, PalletId};
use serde_json::Value;
use sp_genesis_builder::{self, PresetId};
use sp_keyring::Sr25519Keyring;
use sp_runtime::traits::AccountIdConversion;

use crate::{MINING_RESERVE, TOTAL_SUPPLY, UNIT};

/// Get the mining reserve account address
fn mining_reserve_account() -> AccountId {
    PalletId(*b"mineresv").into_account_truncating()
}

/// Community/team allocation
const TEAM_ALLOCATION: u128 = 1_000_000 * UNIT; // 1,000,000 KOD

// Compile-time assertion: Mining reserve + Team allocation = Total supply
const _: () = assert!(
    MINING_RESERVE as u128 + TEAM_ALLOCATION == TOTAL_SUPPLY as u128,
    "Genesis allocations must equal TOTAL_SUPPLY (4,000,000 KOD)"
);

/// Create genesis config with the given endowed accounts
fn kod_genesis(
    endowed_accounts: Vec<(AccountId, u128)>,
    root: AccountId,
) -> Value {
    // Calculate total endowed (should be TEAM_ALLOCATION)
    let total_endowed: u128 = endowed_accounts.iter().map(|(_, b)| b).sum();
    
    // Mining reserve gets the rest
    let mining_reserve_balance = TOTAL_SUPPLY - total_endowed;
    
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
pub fn development_config_genesis() -> Value {
    // Alice gets most of the team allocation for testing
    let endowed = vec![
        (Sr25519Keyring::Alice.to_account_id(), 500_000 * UNIT),
        (Sr25519Keyring::Bob.to_account_id(), 250_000 * UNIT),
        (Sr25519Keyring::Charlie.to_account_id(), 125_000 * UNIT),
        (Sr25519Keyring::Dave.to_account_id(), 62_500 * UNIT),
        (Sr25519Keyring::Eve.to_account_id(), 62_500 * UNIT),
    ];
    
    // Verify allocation sums to TEAM_ALLOCATION
    let total: u128 = endowed.iter().map(|(_, b)| b).sum();
    assert_eq!(total, TEAM_ALLOCATION, "Team allocation must equal 1,000,000 KOD");
    
    kod_genesis(
        endowed,
        Sr25519Keyring::Alice.to_account_id(),
    )
}

/// Local testnet genesis config
pub fn local_config_genesis() -> Value {
    // Distribute among multiple accounts
    let endowed = vec![
        (Sr25519Keyring::Alice.to_account_id(), 500_000 * UNIT),
        (Sr25519Keyring::Bob.to_account_id(), 250_000 * UNIT),
        (Sr25519Keyring::Charlie.to_account_id(), 125_000 * UNIT),
        (Sr25519Keyring::Dave.to_account_id(), 62_500 * UNIT),
        (Sr25519Keyring::Eve.to_account_id(), 62_500 * UNIT),
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
