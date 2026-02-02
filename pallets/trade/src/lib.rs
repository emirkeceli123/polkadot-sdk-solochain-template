//! # Pallet Trade
//!
//! KOD Chain için güvenli ticaret pallet'i.
//! 
//! ## Özellikler
//! - İlan oluşturma (listing)
//! - Satın alma (escrow ile)
//! - Teslimat onayı
//! - Anlaşmazlık açma
//! - KOD-only mod (belirli bloktan sonra sadece KOD ile ticaret)
//!
//! ## Nasıl Çalışır
//! 1. Satıcı ilan verir, teminat (bond) kilitlenir
//! 2. Alıcı satın alır, ödeme + teminat escrow'a gider
//! 3. Teslimat onaylanırsa, satıcıya ödeme yapılır
//! 4. Anlaşmazlık varsa, admin karar verir
//!
//! ## KOD-Only Modu
//! Blok 4,200,000'den sonra (~4 yıl) sadece KOD ile ticaret yapılabilir.
//! Bu, external payment (ETH, BTC vb.) kabul eden ilanları engeller.

// Standart kütüphane yok (WASM için gerekli)
#![cfg_attr(not(feature = "std"), no_std)]

// Pallet'i dışarıya aç
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Saturating;

    // ============================================
    // TİP TANIMLAMALARI
    // ============================================
    
    /// Para birimi tipi (Balance ile çalışmak için)
    pub type BalanceOf<T> = 
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;


    /// İlan durumu
    #[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ListingStatus {
        /// Aktif - satışta
        Active,
        /// Satıldı - escrow'da
        Sold,
        /// İptal edildi
        Cancelled,
        /// Tamamlandı
        Completed,
    }

    /// Ticaret durumu
    #[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum TradeStatus {
        /// Escrow'da bekliyor
        Escrow,
        /// Tamamlandı - ödeme yapıldı
        Completed,
        /// Anlaşmazlık var
        Disputed,
        /// İptal/İade edildi
        Refunded,
    }

    /// İlan bilgisi
    #[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Listing<T: Config> {
        /// Satıcı adresi
        pub seller: T::AccountId,
        /// Fiyat (KOD cinsinden)
        pub price: BalanceOf<T>,
        /// Satıcı teminatı
        pub bond: BalanceOf<T>,
        /// Veri hash'i (IPFS CID veya detayların hash'i)
        pub data_hash: [u8; 32],
        /// External ödeme kabul edilir mi? (true = ETH/BTC/USDT kabul, false = sadece KOD)
        pub accepts_external: bool,
        /// Durum
        pub status: ListingStatus,
        /// Oluşturulma bloğu
        pub created_at: BlockNumberFor<T>,
    }

    /// Ticaret bilgisi
    #[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Trade<T: Config> {
        /// İlan ID
        pub listing_id: u64,
        /// Alıcı adresi
        pub buyer: T::AccountId,
        /// Satıcı adresi
        pub seller: T::AccountId,
        /// Fiyat
        pub price: BalanceOf<T>,
        /// Alıcı teminatı
        pub buyer_bond: BalanceOf<T>,
        /// Satıcı teminatı
        pub seller_bond: BalanceOf<T>,
        /// Durum
        pub status: TradeStatus,
        /// Başlangıç bloğu
        pub created_at: BlockNumberFor<T>,
    }

    // ============================================
    // PALLET YAPILANDIRMASI
    // ============================================

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Pallet ayarları - Runtime bu trait'i implemente eder
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Olay tipi
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        /// Para birimi (pallet-balances kullanır)
        type Currency: ReservableCurrency<Self::AccountId>;

        /// Minimum teminat miktarı
        #[pallet::constant]
        type MinBond: Get<BalanceOf<Self>>;

        /// Maksimum açık ilan sayısı (spam önleme)
        #[pallet::constant]
        type MaxListingsPerUser: Get<u32>;

        /// KOD-only başlangıç bloğu (default ~4 yıl = 4,200,000)
        #[pallet::constant]
        type KodOnlyBlock: Get<BlockNumberFor<Self>>;
    }

    // ============================================
    // STORAGE (Blockchain'de saklanan veriler)
    // ============================================

    /// Sonraki ilan ID'si
    #[pallet::storage]
    #[pallet::getter(fn next_listing_id)]
    pub type NextListingId<T> = StorageValue<_, u64, ValueQuery>;

    /// Sonraki ticaret ID'si
    #[pallet::storage]
    #[pallet::getter(fn next_trade_id)]
    pub type NextTradeId<T> = StorageValue<_, u64, ValueQuery>;

    /// İlanlar: listing_id -> Listing
    #[pallet::storage]
    #[pallet::getter(fn listings)]
    pub type Listings<T: Config> = StorageMap<_, Blake2_128Concat, u64, Listing<T>>;

    /// Ticaretler: trade_id -> Trade
    #[pallet::storage]
    #[pallet::getter(fn trades)]
    pub type Trades<T: Config> = StorageMap<_, Blake2_128Concat, u64, Trade<T>>;

    /// Kullanıcı başına açık ilan sayısı
    #[pallet::storage]
    #[pallet::getter(fn user_listing_count)]
    pub type UserListingCount<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    /// KOD-only blok override (sudo ile değiştirilebilir)
    /// None = Config'deki default değeri kullan
    #[pallet::storage]
    #[pallet::getter(fn kod_only_block_override)]
    pub type KodOnlyBlockOverride<T: Config> = StorageValue<_, BlockNumberFor<T>, OptionQuery>;

    /// Ticaret durdu mu? (acil durum için)
    #[pallet::storage]
    #[pallet::getter(fn trading_paused)]
    pub type TradingPaused<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// Toplam tamamlanan ticaret sayısı
    #[pallet::storage]
    #[pallet::getter(fn total_trades_completed)]
    pub type TotalTradesCompleted<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Toplam işlem hacmi (KOD cinsinden)
    #[pallet::storage]
    #[pallet::getter(fn total_volume)]
    pub type TotalVolume<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    // ============================================
    // EVENTS (Blockchain'de yayınlanan olaylar)
    // ============================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Yeni ilan oluşturuldu
        ListingCreated {
            listing_id: u64,
            seller: T::AccountId,
            price: BalanceOf<T>,
            accepts_external: bool,
        },

        /// İlan iptal edildi
        ListingCancelled {
            listing_id: u64,
            seller: T::AccountId,
        },

        /// Satın alma gerçekleşti (escrow'a alındı)
        Purchased {
            trade_id: u64,
            listing_id: u64,
            buyer: T::AccountId,
            seller: T::AccountId,
            price: BalanceOf<T>,
        },

        /// Ticaret tamamlandı (ödeme yapıldı)
        TradeCompleted {
            trade_id: u64,
            buyer: T::AccountId,
            seller: T::AccountId,
            price: BalanceOf<T>,
        },

        /// Anlaşmazlık açıldı
        DisputeOpened {
            trade_id: u64,
            opener: T::AccountId,
        },

        /// Anlaşmazlık çözüldü
        DisputeResolved {
            trade_id: u64,
            winner: T::AccountId,
        },

        /// İade yapıldı
        Refunded {
            trade_id: u64,
            buyer: T::AccountId,
            amount: BalanceOf<T>,
        },

        /// KOD-only modu aktif oldu
        KodOnlyModeActivated {
            block_number: BlockNumberFor<T>,
        },

        /// KOD-only blok değiştirildi (sudo)
        KodOnlyBlockChanged {
            old_block: BlockNumberFor<T>,
            new_block: BlockNumberFor<T>,
        },

        /// Ticaret durduruldu/başlatıldı
        TradingPausedChanged {
            paused: bool,
        },
    }

    // ============================================
    // ERRORS (Hata mesajları)
    // ============================================

    #[pallet::error]
    pub enum Error<T> {
        /// İlan bulunamadı
        ListingNotFound,
        /// Ticaret bulunamadı
        TradeNotFound,
        /// Yetersiz bakiye
        InsufficientBalance,
        /// Yetersiz teminat
        InsufficientBond,
        /// İlan aktif değil
        ListingNotActive,
        /// Kendi ilanını satın alamazsın
        CannotBuyOwnListing,
        /// Bu işlem için yetkin yok
        NotAuthorized,
        /// Ticaret zaten tamamlandı
        TradeAlreadyCompleted,
        /// Ticaret anlaşmazlık durumunda
        TradeInDispute,
        /// Maksimum ilan sayısına ulaşıldı
        TooManyListings,
        /// Geçersiz durum
        InvalidStatus,
        /// KOD-only modu aktif - external ödeme kabul edilmiyor
        KodOnlyModeActive,
        /// Ticaret şu anda durdurulmuş
        TradingIsPaused,
    }

    // ============================================
    // HELPER FUNCTIONS
    // ============================================

    impl<T: Config> Pallet<T> {
        /// KOD-only modun aktif olup olmadığını kontrol et
        pub fn is_kod_only_active() -> bool {
            let current_block = frame_system::Pallet::<T>::block_number();
            let kod_only_block = Self::get_kod_only_block();
            current_block >= kod_only_block
        }

        /// Efektif KOD-only bloğunu al (override varsa onu kullan)
        pub fn get_kod_only_block() -> BlockNumberFor<T> {
            <KodOnlyBlockOverride<T>>::get().unwrap_or_else(T::KodOnlyBlock::get)
        }
    }

    // ============================================
    // EXTRINSİCS (Kullanıcıların çağırabileceği fonksiyonlar)
    // ============================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Yeni ilan oluştur
        /// 
        /// - `price`: Satış fiyatı
        /// - `bond`: Satıcı teminatı (kilitlenecek)
        /// - `data_hash`: İlan detaylarının hash'i
        /// - `accepts_external`: External ödeme kabul eder mi? (KOD-only mod aktifse false olmalı)
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn create_listing(
            origin: OriginFor<T>,
            price: BalanceOf<T>,
            bond: BalanceOf<T>,
            data_hash: [u8; 32],
            accepts_external: bool,
        ) -> DispatchResult {
            // Ticaret durdurulmuş mu?
            ensure!(!<TradingPaused<T>>::get(), Error::<T>::TradingIsPaused);

            // 1. Kim çağırıyor? (imza kontrolü)
            let seller = ensure_signed(origin)?;

            // 2. KOD-only modu aktifse external ödeme kabul etme
            if Self::is_kod_only_active() && accepts_external {
                return Err(Error::<T>::KodOnlyModeActive.into());
            }

            // 3. Teminat yeterli mi?
            ensure!(bond >= T::MinBond::get(), Error::<T>::InsufficientBond);

            // 4. Kullanıcının çok fazla ilanı var mı?
            let count = UserListingCount::<T>::get(&seller);
            ensure!(count < T::MaxListingsPerUser::get(), Error::<T>::TooManyListings);

            // 5. Teminatı kilitle (reserve)
            T::Currency::reserve(&seller, bond)?;

            // 6. Yeni ilan ID al
            let listing_id = NextListingId::<T>::get();
            NextListingId::<T>::put(listing_id + 1);

            // 7. İlanı kaydet
            let listing = Listing {
                seller: seller.clone(),
                price,
                bond,
                data_hash,
                accepts_external,
                status: ListingStatus::Active,
                created_at: frame_system::Pallet::<T>::block_number(),
            };
            Listings::<T>::insert(listing_id, listing);

            // 8. Kullanıcı ilan sayısını artır
            UserListingCount::<T>::insert(&seller, count + 1);

            // 9. Event yayınla
            Self::deposit_event(Event::ListingCreated {
                listing_id,
                seller,
                price,
                accepts_external,
            });

            Ok(())
        }

        /// İlanı iptal et (sadece satıcı)
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn cancel_listing(
            origin: OriginFor<T>,
            listing_id: u64,
        ) -> DispatchResult {
            let seller = ensure_signed(origin)?;

            // İlanı bul
            let mut listing = Listings::<T>::get(listing_id)
                .ok_or(Error::<T>::ListingNotFound)?;

            // Sadece satıcı iptal edebilir
            ensure!(listing.seller == seller, Error::<T>::NotAuthorized);

            // Sadece aktif ilanlar iptal edilebilir
            ensure!(listing.status == ListingStatus::Active, Error::<T>::InvalidStatus);

            // Teminatı geri ver
            T::Currency::unreserve(&seller, listing.bond);

            // Durumu güncelle
            listing.status = ListingStatus::Cancelled;
            Listings::<T>::insert(listing_id, listing);

            // Kullanıcı ilan sayısını azalt
            UserListingCount::<T>::mutate(&seller, |count| *count = count.saturating_sub(1));

            Self::deposit_event(Event::ListingCancelled { listing_id, seller });

            Ok(())
        }

        /// Satın al - ödeme escrow'a gider
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn purchase(
            origin: OriginFor<T>,
            listing_id: u64,
            buyer_bond: BalanceOf<T>,
        ) -> DispatchResult {
            // Ticaret durdurulmuş mu?
            ensure!(!<TradingPaused<T>>::get(), Error::<T>::TradingIsPaused);

            let buyer = ensure_signed(origin)?;

            // İlanı bul
            let mut listing = Listings::<T>::get(listing_id)
                .ok_or(Error::<T>::ListingNotFound)?;

            // Aktif mi?
            ensure!(listing.status == ListingStatus::Active, Error::<T>::ListingNotActive);

            // KOD-only modu aktifse ve ilan external ödeme kabul ediyorsa, reddet
            if Self::is_kod_only_active() && listing.accepts_external {
                return Err(Error::<T>::KodOnlyModeActive.into());
            }

            // Kendi ilanını alamaz
            ensure!(listing.seller != buyer, Error::<T>::CannotBuyOwnListing);

            // Alıcının yeterli parası var mı? (fiyat + teminat)
            let total_needed = listing.price + buyer_bond;
            ensure!(
                T::Currency::free_balance(&buyer) >= total_needed,
                Error::<T>::InsufficientBalance
            );

            // Parayı kilitle (escrow)
            T::Currency::reserve(&buyer, total_needed)?;

            // Trade oluştur
            let trade_id = NextTradeId::<T>::get();
            NextTradeId::<T>::put(trade_id + 1);

            let trade = Trade {
                listing_id,
                buyer: buyer.clone(),
                seller: listing.seller.clone(),
                price: listing.price,
                buyer_bond,
                seller_bond: listing.bond,
                status: TradeStatus::Escrow,
                created_at: frame_system::Pallet::<T>::block_number(),
            };
            Trades::<T>::insert(trade_id, trade);

            // İlan durumunu güncelle
            listing.status = ListingStatus::Sold;
            Listings::<T>::insert(listing_id, listing.clone());

            Self::deposit_event(Event::Purchased {
                trade_id,
                listing_id,
                buyer,
                seller: listing.seller,
                price: listing.price,
            });

            Ok(())
        }

        /// Teslimatı onayla - alıcı çağırır, satıcıya ödeme yapılır
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn confirm_delivery(
            origin: OriginFor<T>,
            trade_id: u64,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            // Trade'i bul
            let mut trade = Trades::<T>::get(trade_id)
                .ok_or(Error::<T>::TradeNotFound)?;

            // Sadece alıcı onaylayabilir
            ensure!(trade.buyer == caller, Error::<T>::NotAuthorized);

            // Escrow durumunda mı?
            ensure!(trade.status == TradeStatus::Escrow, Error::<T>::InvalidStatus);

            // Alıcının kilidini aç
            T::Currency::unreserve(&trade.buyer, trade.price + trade.buyer_bond);

            // Satıcıya ödeme yap
            T::Currency::transfer(
                &trade.buyer,
                &trade.seller,
                trade.price,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;

            // Satıcının teminatını geri ver
            T::Currency::unreserve(&trade.seller, trade.seller_bond);

            // Durumu güncelle
            trade.status = TradeStatus::Completed;
            Trades::<T>::insert(trade_id, trade.clone());

            // İlanı tamamlandı olarak işaretle
            if let Some(mut listing) = Listings::<T>::get(trade.listing_id) {
                listing.status = ListingStatus::Completed;
                Listings::<T>::insert(trade.listing_id, listing);
            }

            // Kullanıcı ilan sayısını azalt
            UserListingCount::<T>::mutate(&trade.seller, |count| *count = count.saturating_sub(1));

            // İstatistikleri güncelle
            <TotalTradesCompleted<T>>::mutate(|n| *n = n.saturating_add(1));
            <TotalVolume<T>>::mutate(|v| *v = v.saturating_add(trade.price));

            Self::deposit_event(Event::TradeCompleted {
                trade_id,
                buyer: trade.buyer,
                seller: trade.seller,
                price: trade.price,
            });

            Ok(())
        }

        /// Anlaşmazlık aç - alıcı veya satıcı çağırabilir
        #[pallet::call_index(4)]
        #[pallet::weight(10_000)]
        pub fn open_dispute(
            origin: OriginFor<T>,
            trade_id: u64,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            let mut trade = Trades::<T>::get(trade_id)
                .ok_or(Error::<T>::TradeNotFound)?;

            // Alıcı veya satıcı mı?
            ensure!(
                trade.buyer == caller || trade.seller == caller,
                Error::<T>::NotAuthorized
            );

            // Escrow durumunda mı?
            ensure!(trade.status == TradeStatus::Escrow, Error::<T>::InvalidStatus);

            // Durumu güncelle
            trade.status = TradeStatus::Disputed;
            Trades::<T>::insert(trade_id, trade);

            Self::deposit_event(Event::DisputeOpened {
                trade_id,
                opener: caller,
            });

            Ok(())
        }

        /// Anlaşmazlığı çöz - sadece admin (root) çağırabilir
        /// `winner`: true = alıcı kazanır, false = satıcı kazanır
        #[pallet::call_index(5)]
        #[pallet::weight(10_000)]
        pub fn resolve_dispute(
            origin: OriginFor<T>,
            trade_id: u64,
            buyer_wins: bool,
        ) -> DispatchResult {
            // Sadece root (sudo) çağırabilir
            ensure_root(origin)?;

            let mut trade = Trades::<T>::get(trade_id)
                .ok_or(Error::<T>::TradeNotFound)?;

            // Anlaşmazlık durumunda mı?
            ensure!(trade.status == TradeStatus::Disputed, Error::<T>::InvalidStatus);

            if buyer_wins {
                // Alıcı kazandı - iade + satıcı teminatı alıcıya
                T::Currency::unreserve(&trade.buyer, trade.price + trade.buyer_bond);
                
                // Satıcının teminatını alıcıya ver
                T::Currency::repatriate_reserved(
                    &trade.seller,
                    &trade.buyer,
                    trade.seller_bond,
                    frame_support::traits::BalanceStatus::Free,
                )?;

                trade.status = TradeStatus::Refunded;

                Self::deposit_event(Event::Refunded {
                    trade_id,
                    buyer: trade.buyer.clone(),
                    amount: trade.price + trade.seller_bond,
                });

                Self::deposit_event(Event::DisputeResolved {
                    trade_id,
                    winner: trade.buyer.clone(),
                });
            } else {
                // Satıcı kazandı - normal tamamlama + alıcı teminatı satıcıya
                T::Currency::unreserve(&trade.buyer, trade.price + trade.buyer_bond);

                // Satıcıya ödeme + alıcının teminatı
                T::Currency::transfer(
                    &trade.buyer,
                    &trade.seller,
                    trade.price + trade.buyer_bond,
                    frame_support::traits::ExistenceRequirement::AllowDeath,
                )?;

                // Satıcının kendi teminatını geri ver
                T::Currency::unreserve(&trade.seller, trade.seller_bond);

                trade.status = TradeStatus::Completed;

                // İstatistikleri güncelle
                <TotalTradesCompleted<T>>::mutate(|n| *n = n.saturating_add(1));
                <TotalVolume<T>>::mutate(|v| *v = v.saturating_add(trade.price));

                Self::deposit_event(Event::TradeCompleted {
                    trade_id,
                    buyer: trade.buyer.clone(),
                    seller: trade.seller.clone(),
                    price: trade.price,
                });

                Self::deposit_event(Event::DisputeResolved {
                    trade_id,
                    winner: trade.seller.clone(),
                });
            }

            Trades::<T>::insert(trade_id, trade.clone());

            // Kullanıcı ilan sayısını azalt
            UserListingCount::<T>::mutate(&trade.seller, |count| *count = count.saturating_sub(1));

            Ok(())
        }

        // ============================================
        // SUDO FONKSİYONLARI (Admin için)
        // ============================================

        /// KOD-only bloğunu değiştir (sudo only)
        #[pallet::call_index(6)]
        #[pallet::weight(10_000)]
        pub fn set_kod_only_block(
            origin: OriginFor<T>,
            new_block: BlockNumberFor<T>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let old_block = Self::get_kod_only_block();
            <KodOnlyBlockOverride<T>>::put(new_block);

            Self::deposit_event(Event::KodOnlyBlockChanged {
                old_block,
                new_block,
            });

            Ok(())
        }

        /// Ticareti durdur/başlat (sudo only - acil durum için)
        #[pallet::call_index(7)]
        #[pallet::weight(10_000)]
        pub fn set_trading_paused(
            origin: OriginFor<T>,
            paused: bool,
        ) -> DispatchResult {
            ensure_root(origin)?;

            <TradingPaused<T>>::put(paused);

            Self::deposit_event(Event::TradingPausedChanged { paused });

            Ok(())
        }
    }

    // ============================================
    // HOOKS
    // ============================================

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            // KOD-only modu tam olarak bu blokta aktif mi? Event yayınla
            let kod_only_block = Self::get_kod_only_block();
            if n == kod_only_block {
                Self::deposit_event(Event::KodOnlyModeActivated {
                    block_number: n,
                });
                log::info!(
                    target: "trade",
                    "🔒 KOD-only mode activated at block {:?}. External payments no longer accepted.",
                    n
                );
            }
            Weight::zero()
        }
    }
}
