//! # Block Reward Pallet
//!
//! This pallet handles the distribution of mining rewards to block authors.
//! When a miner successfully mines a block, they receive 1000 KOD tokens.
//!
//! ## Overview
//!
//! - Block rewards are paid from a "Mining Reserve" account funded at genesis
//! - Each block author receives `RewardAmount` (1000 KOD) tokens
//! - The miner's address is passed via inherent data from the node
//! - Rewards stop when the Mining Reserve is depleted

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

        /// The amount of tokens to reward for each block (1000 KOD = 1000 * 10^18 units).
        #[pallet::constant]
        type RewardAmount: Get<BalanceOf<Self>>;

        /// The account that holds the mining reserve funds.
        #[pallet::constant]
        type MiningReserveAccount: Get<Self::AccountId>;
    }

    /// Storage for the current block's miner address (set via inherent)
    #[pallet::storage]
    #[pallet::getter(fn pending_miner)]
    pub type PendingMiner<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A block reward was successfully paid. [miner, amount]
        RewardPaid { miner: T::AccountId, amount: BalanceOf<T> },
        /// Mining reserve is depleted - no more rewards available.
        ReserveExhausted,
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The miner address provided is invalid.
        InvalidMinerAddress,
        /// Insufficient funds in mining reserve.
        InsufficientReserve,
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
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(_n: BlockNumberFor<T>) {
            // Pay the block reward if a miner was set
            if let Some(miner) = <PendingMiner<T>>::take() {
                let reward = T::RewardAmount::get();
                let reserve = T::MiningReserveAccount::get();

                // Check if reserve has enough funds
                let reserve_balance = T::Currency::free_balance(&reserve);
                if reserve_balance < reward {
                    log::warn!(
                        target: "mining",
                        "⚠️ Mining reserve exhausted! No more block rewards available."
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
                        log::info!(
                            target: "mining",
                            "💰 Block reward of {:?} paid to miner {:?}",
                            reward,
                            miner
                        );
                        Self::deposit_event(Event::RewardPaid { miner, amount: reward });
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

