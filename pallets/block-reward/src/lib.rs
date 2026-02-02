//! # Block Reward Pallet
//!
//! This pallet handles the distribution of mining rewards to block authors.
//! Rewards follow a halving schedule, starting at 250 KOD and halving every ~2 years.
//!
//! ## Overview
//!
//! - Block rewards are paid from a "Mining Reserve" account funded at genesis
//! - Initial reward: 250 KOD per block
//! - Halving: Every 2,100,000 blocks (~2 years at 30s/block)
//! - The miner's address is passed via inherent data from the node
//! - Rewards stop when the Mining Reserve is depleted or after ~10 halvings
//!
//! ## Halving Schedule (at 30s blocks)
//!
//! | Era | Blocks | Reward | Total Mined | Cumulative |
//! |-----|--------|--------|-------------|------------|
//! | 1   | 0-2.1M | 250    | 525M        | 525M       |
//! | 2   | 2.1M-4.2M | 125 | 262.5M      | 787.5M     |
//! | 3   | 4.2M-6.3M | 62  | 130.2M      | 917.7M     |
//! | 4   | 6.3M-8.4M | 31  | 65.1M       | 982.8M     |
//! | ... | ...    | ...    | ...         | ~1B        |

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "std")]
pub mod inherent;

extern crate alloc;
use alloc::vec::Vec;
use codec::{Decode, Encode};
use frame_support::traits::Currency;
use scale_info::TypeInfo;

/// The inherent identifier for block reward data
pub const INHERENT_IDENTIFIER: sp_inherents::InherentIdentifier = *b"blkrewrd";

/// Inherent data: the miner's account ID (SS58 encoded string)
#[derive(Clone, Encode, Decode, TypeInfo, Debug, PartialEq, Eq)]
pub struct MinerInherentData {
    /// The SS58-encoded address of the miner to receive the reward
    pub miner_address: Option<Vec<u8>>,
    /// Block number (for validation)
    pub block_number: u32,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{Zero, Saturating};

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The currency mechanism for rewards.
        type Currency: Currency<Self::AccountId>;

        /// Initial block reward (250 KOD = 250 * 10^18 units).
        /// This value halves every HalvingInterval blocks.
        #[pallet::constant]
        type RewardAmount: Get<BalanceOf<Self>>;

        /// Number of blocks between halvings (~2,100,000 for ~2 years at 30s/block)
        #[pallet::constant]
        type HalvingInterval: Get<BlockNumberFor<Self>>;

        /// The account that holds the mining reserve funds.
        #[pallet::constant]
        type MiningReserveAccount: Get<Self::AccountId>;
    }

    /// Storage for the current block's miner address (set via inherent)
    #[pallet::storage]
    #[pallet::getter(fn pending_miner)]
    pub type PendingMiner<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    /// Override for block reward (sudo can set this to change rewards)
    /// If Some, this value is used instead of the halving calculation
    #[pallet::storage]
    #[pallet::getter(fn reward_override)]
    pub type RewardOverride<T: Config> = StorageValue<_, BalanceOf<T>, OptionQuery>;

    /// Emergency pause for mining rewards
    #[pallet::storage]
    #[pallet::getter(fn rewards_paused)]
    pub type RewardsPaused<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// Total rewards paid out (for tracking)
    #[pallet::storage]
    #[pallet::getter(fn total_rewards_paid)]
    pub type TotalRewardsPaid<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A block reward was successfully paid. [miner, amount, block_number]
        RewardPaid { 
            miner: T::AccountId, 
            amount: BalanceOf<T>,
            block_number: BlockNumberFor<T>,
        },
        /// Mining reserve is depleted - no more rewards available.
        ReserveExhausted,
        /// Halving occurred - reward amount changed
        HalvingOccurred {
            block_number: BlockNumberFor<T>,
            new_reward: BalanceOf<T>,
            era: u32,
        },
        /// Reward override was set by sudo
        RewardOverrideSet { new_reward: Option<BalanceOf<T>> },
        /// Rewards were paused/unpaused
        RewardsPausedChanged { paused: bool },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The miner address provided is invalid.
        InvalidMinerAddress,
        /// Insufficient funds in mining reserve.
        InsufficientReserve,
        /// Rewards are currently paused.
        RewardsPaused,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Set the miner address for block reward via inherent.
        /// This is called automatically by the block author.
        #[pallet::call_index(0)]
        #[pallet::weight((0, DispatchClass::Mandatory))]
        pub fn set_miner(
            origin: OriginFor<T>,
            miner: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            ensure_none(origin)?;
            <PendingMiner<T>>::put(miner);
            Ok(Pays::No.into())
        }

        /// Set a reward override (sudo only).
        /// Pass None to return to normal halving schedule.
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn set_reward_override(
            origin: OriginFor<T>,
            new_reward: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            if let Some(reward) = new_reward {
                <RewardOverride<T>>::put(reward);
            } else {
                <RewardOverride<T>>::kill();
            }
            
            Self::deposit_event(Event::RewardOverrideSet { new_reward });
            Ok(())
        }

        /// Pause or unpause mining rewards (sudo only).
        /// Use in emergencies.
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn set_rewards_paused(
            origin: OriginFor<T>,
            paused: bool,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            <RewardsPaused<T>>::put(paused);
            Self::deposit_event(Event::RewardsPausedChanged { paused });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Calculate the current block reward based on halving schedule
        pub fn calculate_reward(block_number: BlockNumberFor<T>) -> BalanceOf<T> {
            // If there's an override, use it
            if let Some(override_reward) = <RewardOverride<T>>::get() {
                return override_reward;
            }

            let halving_interval = T::HalvingInterval::get();
            let initial_reward = T::RewardAmount::get();

            // Avoid division by zero
            if halving_interval.is_zero() {
                return initial_reward;
            }

            // Calculate which era we're in (0 = first era)
            let era: u32 = (block_number / halving_interval)
                .try_into()
                .unwrap_or(u32::MAX);

            // After 10 halvings, reward is essentially zero
            if era >= 10 {
                return Zero::zero();
            }

            // Divide by 2^era (right shift)
            // We need to convert to u128 for the shift operation
            let initial_u128: u128 = initial_reward.try_into().unwrap_or(0);
            let reward_u128 = initial_u128 >> era;
            
            reward_u128.try_into().unwrap_or_else(|_| Zero::zero())
        }

        /// Get the current halving era (0 = first era)
        pub fn current_era() -> u32 {
            let current_block = frame_system::Pallet::<T>::block_number();
            let halving_interval = T::HalvingInterval::get();
            
            if halving_interval.is_zero() {
                return 0;
            }
            
            (current_block / halving_interval)
                .try_into()
                .unwrap_or(0)
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(n: BlockNumberFor<T>) {
            // Check if rewards are paused
            if <RewardsPaused<T>>::get() {
                log::debug!(target: "mining", "⏸️ Mining rewards are paused");
                return;
            }

            // Pay the block reward if a miner was set
            if let Some(miner) = <PendingMiner<T>>::take() {
                let reward = Self::calculate_reward(n);
                let reserve = T::MiningReserveAccount::get();

                // If reward is zero (after many halvings), skip
                if reward.is_zero() {
                    log::info!(
                        target: "mining",
                        "🔚 Block reward is zero after halvings. Miners earn from tx fees only."
                    );
                    return;
                }

                // Check if reserve has enough funds
                let reserve_balance = T::Currency::free_balance(&reserve);
                if reserve_balance < reward {
                    log::warn!(
                        target: "mining",
                        "⚠️ Mining reserve exhausted! Balance: {:?}, Needed: {:?}",
                        reserve_balance,
                        reward
                    );
                    Self::deposit_event(Event::ReserveExhausted);
                    return;
                }

                // Transfer reward from reserve to miner
                match T::Currency::transfer(
                    &reserve,
                    &miner,
                    reward,
                    frame_support::traits::ExistenceRequirement::KeepAlive,
                ) {
                    Ok(_) => {
                        // Update total rewards paid
                        let total = <TotalRewardsPaid<T>>::get();
                        <TotalRewardsPaid<T>>::put(total.saturating_add(reward));

                        log::info!(
                            target: "mining",
                            "💰 Block #{:?} reward: {:?} paid to miner",
                            n,
                            reward
                        );
                        Self::deposit_event(Event::RewardPaid { 
                            miner, 
                            amount: reward,
                            block_number: n,
                        });

                        // Check if halving just occurred (first block of new era)
                        let halving_interval = T::HalvingInterval::get();
                        if !halving_interval.is_zero() && n % halving_interval == Zero::zero() && !n.is_zero() {
                            let era = Self::current_era();
                            let new_reward = Self::calculate_reward(n);
                            log::info!(
                                target: "mining",
                                "🔔 HALVING! Era {} started. New reward: {:?}",
                                era,
                                new_reward
                            );
                            Self::deposit_event(Event::HalvingOccurred {
                                block_number: n,
                                new_reward,
                                era,
                            });
                        }
                    }
                    Err(e) => {
                        log::error!(
                            target: "mining",
                            "❌ Failed to pay block reward: {:?}",
                            e
                        );
                    }
                }
            }
        }
    }

    #[pallet::inherent]
    impl<T: Config> ProvideInherent for Pallet<T>
    where
        T::AccountId: Decode,
    {
        type Call = Call<T>;
        type Error = sp_inherents::MakeFatalError<()>;

        const INHERENT_IDENTIFIER: sp_inherents::InherentIdentifier = INHERENT_IDENTIFIER;

        fn create_inherent(data: &sp_inherents::InherentData) -> Option<Self::Call> {
            let inherent_data = data
                .get_data::<MinerInherentData>(&INHERENT_IDENTIFIER)
                .ok()
                .flatten()?;

            // Decode the miner address from SS58 bytes
            if let Some(address_bytes) = inherent_data.miner_address {
                // Try to decode as AccountId directly
                if let Ok(miner) = T::AccountId::decode(&mut &address_bytes[..]) {
                    return Some(Call::set_miner { miner });
                }
                
                // If direct decode fails, try parsing as SS58 string
                if let Ok(address_str) = core::str::from_utf8(&address_bytes) {
                    use sp_core::crypto::Ss58Codec;
                    if let Ok(account_id) = sp_core::crypto::AccountId32::from_ss58check(address_str) {
                        let account_bytes: [u8; 32] = account_id.into();
                        if let Ok(miner) = T::AccountId::decode(&mut &account_bytes[..]) {
                            return Some(Call::set_miner { miner });
                        }
                    }
                }
                
                log::warn!(
                    target: "mining",
                    "⚠️ Failed to decode miner address from inherent data"
                );
            }

            None
        }

        fn is_inherent(call: &Self::Call) -> bool {
            matches!(call, Call::set_miner { .. })
        }

        fn check_inherent(
            _call: &Self::Call,
            _data: &sp_inherents::InherentData,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }
}
