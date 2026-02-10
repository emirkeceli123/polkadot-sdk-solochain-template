//! KOD Chain Runtime
//!
//! A Proof-of-Work blockchain with:
//! - 1,000,000,000 KOD total supply cap (1 Billion)
//! - 50 KOD initial block reward (halves every ~2 years)
//! - 6 second target block time
//! - SHA3-256 mining algorithm
//! - KOD-only trading after block 21,000,000 (~4 years)

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod apis;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod configs;
pub mod genesis_config_presets;

extern crate alloc;
use alloc::vec::Vec;
use sp_runtime::{
    generic,
    traits::{BlakeTwo256, IdentifyAccount, Verify},
    MultiAddress, MultiSignature,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

// ============================================================================
// ECONOMIC CONSTANTS
// ============================================================================

/// Total supply of KOD tokens: 1,000,000,000 KOD (1 Billion)
/// All tokens are allocated at genesis (no inflation)
pub const TOTAL_SUPPLY: Balance = 1_000_000_000 * UNIT;

/// Initial mining reward per block: 50 KOD
/// This halves every HALVING_INTERVAL blocks
/// (Was 250 KOD at 30s blocks; adjusted to 50 KOD for 6s blocks to keep same annual emission)
pub const INITIAL_BLOCK_REWARD: Balance = 50 * UNIT;

/// Halving interval: every 10,500,000 blocks (~2 years at 6s/block)
/// After each interval, block reward is halved
/// (Was 2,100,000 at 30s blocks; 2,100,000 × 5 = 10,500,000)
pub const HALVING_INTERVAL: BlockNumber = 10_500_000;

/// KOD-only trading block: after block 21,000,000 (~4 years)
/// Only KOD can be used for trading after this block
/// (Was 4,200,000 at 30s blocks; 4,200,000 × 5 = 21,000,000)
pub const KOD_ONLY_BLOCK: BlockNumber = 21_000_000;

/// Mining reserve allocation: 900,000,000 KOD (900M for ~8 years of rewards)
pub const MINING_RESERVE: Balance = 900_000_000 * UNIT;

/// Test/Foundation allocation: 100,000,000 KOD (100M)
pub const FOUNDATION_RESERVE: Balance = 100_000_000 * UNIT;

// ============================================================================
// UNIT DEFINITIONS
// ============================================================================

/// 1 KOD = 10^18 smallest units (like ETH/wei)
pub const UNIT: Balance = 1_000_000_000_000_000_000;
pub const MILLI_UNIT: Balance = UNIT / 1_000;
pub const MICRO_UNIT: Balance = UNIT / 1_000_000;

/// Existential deposit: minimum balance to keep account alive
pub const EXISTENTIAL_DEPOSIT: Balance = MILLI_UNIT;

// ============================================================================
// BLOCK TIME
// ============================================================================

/// Target block time: 6 seconds
pub const MILLI_SECS_PER_BLOCK: u64 = 6_000;
pub const SLOT_DURATION: u64 = MILLI_SECS_PER_BLOCK;

// Time is measured by number of blocks
pub const MINUTES: BlockNumber = 60_000 / (MILLI_SECS_PER_BLOCK as BlockNumber); // 10 blocks/min
pub const HOURS: BlockNumber = MINUTES * 60;   // 600 blocks/hour
pub const DAYS: BlockNumber = HOURS * 24;      // 14,400 blocks/day
pub const YEARS: BlockNumber = DAYS * 365;     // ~5,256,000 blocks/year

// ============================================================================
// HALVING CALCULATION
// ============================================================================

/// Calculate the block reward for a given block number
/// Reward halves every HALVING_INTERVAL blocks
pub fn calculate_block_reward(block_number: BlockNumber) -> Balance {
    let halvings = block_number / HALVING_INTERVAL;
    
    // After 10 halvings, reward is essentially zero (50 / 1024 < 1)
    if halvings >= 10 {
        return 0;
    }
    
    // Shift right by halvings (divide by 2^halvings)
    INITIAL_BLOCK_REWARD >> halvings
}

pub const BLOCK_HASH_COUNT: BlockNumber = 2400;

// ============================================================================
// SESSION KEYS (Minimal for PoW - no actual keys needed)
// ============================================================================

use sp_runtime::impl_opaque_keys;

impl_opaque_keys! {
    pub struct SessionKeys {}
}

// ============================================================================
// OPAQUE TYPES
// ============================================================================

/// Opaque types for use by the node. These don't need to know runtime specifics.
pub mod opaque {
    use super::*;
    use sp_runtime::{
        generic,
        traits::{BlakeTwo256, Hash as HashT},
    };

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;
    /// Opaque block hash type.
    pub type Hash = <BlakeTwo256 as HashT>::Output;
}

// ============================================================================
// VERSION INFO
// ============================================================================

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: alloc::borrow::Cow::Borrowed("kod-chain"),
    impl_name: alloc::borrow::Cow::Borrowed("kod-chain"),
    authoring_version: 1,
    spec_version: 201,  // Bumped for 6s block time
    impl_version: 1,
    apis: apis::RUNTIME_API_VERSIONS,
    transaction_version: 1,
    system_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

// ============================================================================
// TYPE ALIASES
// ============================================================================

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The `TransactionExtension` to the basic transaction logic.
pub type TxExtension = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
    frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
    frame_system::WeightReclaim<Runtime>,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, TxExtension>;

/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, TxExtension>;

/// All migrations of the runtime, aside from the ones declared in the pallets.
#[allow(unused_parens)]
type Migrations = ();

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
    Migrations,
>;

// ============================================================================
// RUNTIME CONSTRUCTION
// ============================================================================

/// Create the runtime by composing the FRAME pallets.
/// Note: No Aura or Grandpa - this is a PoW chain!
#[frame_support::runtime]
mod runtime {
    #[runtime::runtime]
    #[runtime::derive(
        RuntimeCall,
        RuntimeEvent,
        RuntimeError,
        RuntimeOrigin,
        RuntimeFreezeReason,
        RuntimeHoldReason,
        RuntimeSlashReason,
        RuntimeLockId,
        RuntimeTask,
        RuntimeViewFunction
    )]
    pub struct Runtime;

    #[runtime::pallet_index(0)]
    pub type System = frame_system;

    #[runtime::pallet_index(1)]
    pub type Timestamp = pallet_timestamp;

    #[runtime::pallet_index(2)]
    pub type Balances = pallet_balances;

    #[runtime::pallet_index(3)]
    pub type TransactionPayment = pallet_transaction_payment;

    #[runtime::pallet_index(4)]
    pub type Sudo = pallet_sudo;

    #[runtime::pallet_index(5)]
    pub type BlockReward = pallet_block_reward;

    #[runtime::pallet_index(6)]
    pub type Template = pallet_template;

    #[runtime::pallet_index(7)]
    pub type Trade = pallet_trade;
}
