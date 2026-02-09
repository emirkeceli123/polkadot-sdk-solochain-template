//! # Pallet Trade
//!
//! KOD Chain için güvenli ticaret pallet'i - 4'lü İmza Sistemi.
//! 
//! ## Özellikler
//! - İlan oluşturma (listing) + Cihaz attestation
//! - Satın alma (escrow ile) + Merkle proof
//! - Teslimat onayı + Çoklu imza
//! - Anlaşmazlık açma + Kanıt sunma
//! - KOD-only mod (belirli bloktan sonra sadece KOD ile ticaret)
//!
//! ## 4'lü İmza Sistemi
//! 1. Satıcı: "Bu koşullarla satıyorum"
//! 2. Cihaz: "Bu veriler benden çıktı" (device attestation)
//! 3. Alıcı: "Kabul ediyorum / Onaylıyorum"
//! 4. Madenciler: "Bloğa yazdık"
//!
//! ## Nasıl Çalışır
//! 1. Satıcı ilan verir + cihaz attestation + teminat kilitlenir
//! 2. Alıcı satın alır, ödeme + teminat escrow'a gider
//! 3. Buluşmada cihaz tekrar doğrulanır
//! 4. Teslimat onaylanırsa, satıcıya ödeme yapılır
//! 5. Anlaşmazlıkta Merkle proof ile kanıt sunulur
//!
//! ## KOD-Only Modu
//! Blok 4,200,000'den sonra (~4 yıl) sadece KOD ile ticaret yapılabilir.

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
    use sp_io::hashing::blake2_256;

    // ============================================
    // TİP TANIMLAMALARI
    // ============================================
    
    /// Para birimi tipi (Balance ile çalışmak için)
    pub type BalanceOf<T> = 
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// Maksimum Merkle proof derinliği
    pub const MAX_MERKLE_PROOF_DEPTH: u32 = 16;

    /// Maksimum tanılama test sayısı
    pub const MAX_DIAGNOSTIC_TESTS: u32 = 16;

    // ============================================
    // TANITLAMA (DIAGNOSTICS) TİPLERİ
    // ============================================

    /// Tek bir test sonucu (on-chain)
    #[derive(Clone, Encode, Decode, DecodeWithMemTracking, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum DiagTestResult {
        /// Test geçti
        Passed,
        /// Test başarısız
        Failed,
        /// Test atlandı
        Skipped,
    }

    /// Tek bir tanılama test kaydı (on-chain)
    /// Her test ayrı ayrı blockchain'e yazılır
    #[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct DiagTestEntry {
        /// Test kimliği hash'i (blake2("screen_colors"), blake2("touch_grid") vb.)
        pub test_id_hash: [u8; 32],
        /// Test sonucu
        pub result: DiagTestResult,
        /// Test detay hash'i (açıklama metni hash'i, off-chain doğrulanır)
        pub detail_hash: [u8; 32],
    }

    /// Tanılama raporu (on-chain, trade'e bağlı)
    #[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct DiagnosticReport<T: Config> {
        /// Raporu gönderen (satıcı veya alıcı)
        pub submitter: T::AccountId,
        /// Cihaz model hash'i (blake2(model_name))
        pub device_model_hash: [u8; 32],
        /// Cihaz üretici hash'i (blake2(manufacturer))
        pub device_manufacturer_hash: [u8; 32],
        /// İşletim sistemi hash'i (blake2("Android 16"))
        pub os_hash: [u8; 32],
        /// Test sayısı
        pub test_count: u32,
        /// Geçen test sayısı
        pub passed_count: u32,
        /// Başarısız test sayısı
        pub failed_count: u32,
        /// Genel skor (0-100)
        pub score: u8,
        /// Tüm rapor verilerinin birleşik hash'i (off-chain tam rapor doğrulaması)
        pub report_hash: [u8; 32],
        /// Raporun sunulduğu blok
        pub submitted_at: BlockNumberFor<T>,
    }

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
        /// Satıcı onayı bekleniyor (alıcı satın aldı, satıcı henüz kabul etmedi)
        PendingSellerConfirm,
        /// Escrow'da bekliyor (satıcı kabul etti, para kilitli, teslimat bekleniyor)
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
        /// Koşulların Merkle root'u (off-chain JSON'un hash ağacı)
        pub conditions_root: [u8; 32],
        /// IPFS CID hash'i (tam CID off-chain'de saklanır)
        pub ipfs_cid_hash: Option<[u8; 32]>,
        /// Cihaz attestation hash'i (tam attestation off-chain'de)
        /// Hash = blake2(model_id || data_hash || signature || public_key || timestamp)
        pub device_attestation_hash: Option<[u8; 32]>,
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
        /// Anlaşma hash'i (conditions_root + "accepted" + timestamp)
        pub contract_hash: [u8; 32],
        /// Teslimat cihaz attestation hash'i (buluşmada alınan, tam veri off-chain)
        pub delivery_attestation_hash: Option<[u8; 32]>,
        /// Son hash (teslimat onayı sonrası)
        pub final_hash: Option<[u8; 32]>,
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

    /// Sunulan kanıtlar: (trade_id, proof_index) -> (condition_hash, submitter, block)
    /// Anlaşmazlıkta sunulan Merkle proof'ları saklar (sadece hash'ler)
    #[pallet::storage]
    #[pallet::getter(fn submitted_proofs)]
    pub type SubmittedProofsSimple<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, u64,           // trade_id
        Blake2_128Concat, u32,           // proof_index
        (
            [u8; 32],                      // condition_hash
            T::AccountId,                  // submitter
            BlockNumberFor<T>,             // block_number
        ),
    >;

    /// Ticaret başına sunulan proof sayısı
    #[pallet::storage]
    #[pallet::getter(fn proof_count)]
    pub type ProofCount<T: Config> = StorageMap<_, Blake2_128Concat, u64, u32, ValueQuery>;

    /// Tanılama raporları: trade_id -> DiagnosticReport
    #[pallet::storage]
    #[pallet::getter(fn diagnostic_reports)]
    pub type DiagnosticReports<T: Config> = StorageMap<_, Blake2_128Concat, u64, DiagnosticReport<T>>;

    /// Tanılama test detayları: (trade_id, test_index) -> DiagTestEntry
    /// Her test sonucu ayrı ayrı sorgulanabilir
    #[pallet::storage]
    #[pallet::getter(fn diagnostic_tests)]
    pub type DiagnosticTests<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, u64,    // trade_id
        Blake2_128Concat, u32,    // test_index
        DiagTestEntry,
    >;

    /// Trade için tanılama sunulmuş mu?
    #[pallet::storage]
    #[pallet::getter(fn has_diagnostics)]
    pub type HasDiagnostics<T: Config> = StorageMap<_, Blake2_128Concat, u64, bool, ValueQuery>;

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

        /// Cihaz attestation eklendi
        DeviceAttestationAdded {
            listing_id: u64,
            attestation_hash: [u8; 32],
        },

        /// Teslimat attestation eklendi
        DeliveryAttestationAdded {
            trade_id: u64,
            attestation_hash: [u8; 32],
        },

        /// Merkle proof doğrulandı (anlaşmazlıkta kanıt)
        MerkleProofVerified {
            trade_id: u64,
            condition_hash: [u8; 32],
            verified: bool,
        },

        /// Koşul kanıtı sunuldu
        ConditionProofSubmitted {
            trade_id: u64,
            submitter: T::AccountId,
            condition_hash: [u8; 32],
        },

        /// Tanılama raporu sunuldu (tüm test sonuçları blockchain'de)
        DiagnosticReportSubmitted {
            trade_id: u64,
            submitter: T::AccountId,
            device_model_hash: [u8; 32],
            score: u8,
            passed_count: u32,
            failed_count: u32,
            report_hash: [u8; 32],
        },

        /// Tek bir tanılama testi kaydedildi
        DiagnosticTestRecorded {
            trade_id: u64,
            test_index: u32,
            test_id_hash: [u8; 32],
            result: DiagTestResult,
        },

        /// Satıcı ticareti kabul etti (PendingSellerConfirm -> Escrow)
        TradeAccepted {
            trade_id: u64,
            seller: T::AccountId,
            buyer: T::AccountId,
            price: BalanceOf<T>,
            accepted_at: BlockNumberFor<T>,
        },

        /// Satıcı ticareti reddetti (PendingSellerConfirm -> Refunded)
        TradeRejected {
            trade_id: u64,
            seller: T::AccountId,
            buyer: T::AccountId,
            reason_hash: Option<[u8; 32]>,
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
        /// Geçersiz Merkle proof
        InvalidMerkleProof,
        /// Proof zaten sunulmuş
        ProofAlreadySubmitted,
        /// Geçersiz cihaz imzası
        InvalidDeviceSignature,
        /// Cihaz attestation eksik
        MissingDeviceAttestation,
        /// Cihaz verisi çok büyük
        DeviceDataTooLarge,
        /// IPFS CID çok uzun
        IpfsCidTooLong,
        /// Merkle proof çok derin
        MerkleProofTooDeep,
        /// Çok fazla tanılama testi
        TooManyDiagnosticTests,
        /// Tanılama raporu zaten sunulmuş
        DiagnosticsAlreadySubmitted,
        /// Tanılama raporu gerekli (confirmDelivery için)
        DiagnosticsRequired,
        /// Ticaret satıcı onayı beklemiyor
        NotPendingSellerConfirm,
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

        /// Merkle proof doğrulama
        /// 
        /// - `root`: Beklenen Merkle root
        /// - `leaf`: Doğrulanacak yaprak veri
        /// - `proof`: Merkle proof (sibling hash'ler)
        /// - `index`: Yaprak index'i
        /// 
        /// Returns: proof geçerli mi?
        pub fn verify_merkle_proof(
            root: [u8; 32],
            leaf: &[u8],
            proof: &[[u8; 32]],
            index: u32,
        ) -> bool {
            let mut computed_hash = blake2_256(leaf);
            let mut current_index = index;

            for proof_element in proof.iter() {
                computed_hash = if current_index % 2 == 0 {
                    // Sol taraf - proof sağda
                    let combined = [computed_hash.as_slice(), proof_element.as_slice()].concat();
                    blake2_256(&combined)
                } else {
                    // Sağ taraf - proof solda
                    let combined = [proof_element.as_slice(), computed_hash.as_slice()].concat();
                    blake2_256(&combined)
                };
                current_index /= 2;
            }

            computed_hash == root
        }

        /// Contract hash hesapla (anlaşma için)
        pub fn compute_contract_hash(
            conditions_root: [u8; 32],
            buyer: &T::AccountId,
            timestamp: u64,
        ) -> [u8; 32] {
            let buyer_bytes = buyer.encode();
            let timestamp_bytes = timestamp.to_le_bytes();
            let combined = [
                conditions_root.as_slice(),
                buyer_bytes.as_slice(),
                timestamp_bytes.as_slice(),
                b"accepted",
            ].concat();
            blake2_256(&combined)
        }

        /// Final hash hesapla (teslimat onayı için)
        pub fn compute_final_hash(
            contract_hash: [u8; 32],
            delivery_data_hash: Option<[u8; 32]>,
        ) -> [u8; 32] {
            let delivery_bytes = delivery_data_hash.unwrap_or([0u8; 32]);
            let combined = [
                contract_hash.as_slice(),
                delivery_bytes.as_slice(),
                b"completed",
            ].concat();
            blake2_256(&combined)
        }

        /// Final hash hesapla - tanılama raporu dahil
        /// Hash = blake2(contract_hash + delivery_hash + diagnostics_hash + "completed")
        /// Bu hash ile sonradan tüm süreç doğrulanabilir
        pub fn compute_final_hash_with_diagnostics(
            contract_hash: [u8; 32],
            delivery_data_hash: Option<[u8; 32]>,
            diagnostics_hash: Option<[u8; 32]>,
        ) -> [u8; 32] {
            let delivery_bytes = delivery_data_hash.unwrap_or([0u8; 32]);
            let diagnostics_bytes = diagnostics_hash.unwrap_or([0u8; 32]);
            let combined = [
                contract_hash.as_slice(),
                delivery_bytes.as_slice(),
                diagnostics_bytes.as_slice(),
                b"completed_with_diagnostics",
            ].concat();
            blake2_256(&combined)
        }

        /// Merkle proof doğrulama (hash zaten hesaplanmış)
        /// 
        /// - `root`: Beklenen Merkle root
        /// - `leaf_hash`: Yaprak verinin hash'i
        /// - `proof`: Merkle proof (sibling hash'ler)
        /// - `index`: Yaprak index'i
        pub fn verify_merkle_proof_hashed(
            root: [u8; 32],
            leaf_hash: [u8; 32],
            proof: &[[u8; 32]],
            index: u32,
        ) -> bool {
            let mut computed_hash = leaf_hash;
            let mut current_index = index;

            for proof_element in proof.iter() {
                computed_hash = if current_index % 2 == 0 {
                    // Sol taraf - proof sağda
                    let combined = [computed_hash.as_slice(), proof_element.as_slice()].concat();
                    blake2_256(&combined)
                } else {
                    // Sağ taraf - proof solda
                    let combined = [proof_element.as_slice(), computed_hash.as_slice()].concat();
                    blake2_256(&combined)
                };
                current_index /= 2;
            }

            computed_hash == root
        }
    }

    // ============================================
    // EXTRINSİCS (Kullanıcıların çağırabileceği fonksiyonlar)
    // ============================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Yeni ilan oluştur (Merkle proof + Cihaz Attestation destekli)
        /// 
        /// - `price`: Satış fiyatı (KOD cinsinden)
        /// - `bond`: Satıcı teminatı (kilitlenecek)
        /// - `conditions_root`: Koşulların Merkle root'u
        /// - `ipfs_cid_hash`: Detaylı koşullar için IPFS CID hash'i (opsiyonel)
        /// - `device_attestation_hash`: Cihaz attestation hash'i (opsiyonel, tam veri off-chain)
        /// - `accepts_external`: External ödeme kabul eder mi?
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn create_listing(
            origin: OriginFor<T>,
            price: BalanceOf<T>,
            bond: BalanceOf<T>,
            conditions_root: [u8; 32],
            ipfs_cid_hash: Option<[u8; 32]>,
            device_attestation_hash: Option<[u8; 32]>,
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
                conditions_root,
                ipfs_cid_hash,
                device_attestation_hash,
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
                seller: seller.clone(),
                price,
                accepts_external,
            });

            // 10. Eğer device attestation hash varsa, ek event
            if let Some(attestation_hash) = device_attestation_hash {
                Self::deposit_event(Event::DeviceAttestationAdded {
                    listing_id,
                    attestation_hash,
                });
            }

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
        /// Alıcı koşulları kabul ettiğinde contract_hash oluşturulur
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

            // Contract hash hesapla (alıcı koşulları kabul ettiğini kanıtlar)
            // Timestamp için blok numarasını kullanıyoruz (offchain erişim yok)
            let current_block = frame_system::Pallet::<T>::block_number();
            let current_timestamp: u64 = current_block.try_into().unwrap_or(0);
            let contract_hash = Self::compute_contract_hash(
                listing.conditions_root,
                &buyer,
                current_timestamp,
            );

            let trade = Trade {
                listing_id,
                buyer: buyer.clone(),
                seller: listing.seller.clone(),
                price: listing.price,
                buyer_bond,
                seller_bond: listing.bond,
                contract_hash,
                delivery_attestation_hash: None,
                final_hash: None,
                status: TradeStatus::PendingSellerConfirm, // Satıcı onayı bekleniyor
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

        /// Satıcı ticareti kabul eder
        /// PendingSellerConfirm -> Escrow durumuna geçirir
        /// Taraflar ve şartlar blockchain'e yazılır
        #[pallet::call_index(10)]
        #[pallet::weight(10_000)]
        pub fn accept_trade(
            origin: OriginFor<T>,
            trade_id: u64,
        ) -> DispatchResult {
            let seller = ensure_signed(origin)?;

            // Ticaret var mı?
            let mut trade = Trades::<T>::get(trade_id)
                .ok_or(Error::<T>::TradeNotFound)?;

            // Satıcı mı çağırıyor?
            ensure!(trade.seller == seller, Error::<T>::NotAuthorized);

            // PendingSellerConfirm durumunda mı?
            ensure!(
                trade.status == TradeStatus::PendingSellerConfirm,
                Error::<T>::NotPendingSellerConfirm
            );

            // Durumu Escrow'a geçir
            trade.status = TradeStatus::Escrow;
            Trades::<T>::insert(trade_id, trade.clone());

            let now = frame_system::Pallet::<T>::block_number();

            Self::deposit_event(Event::TradeAccepted {
                trade_id,
                seller,
                buyer: trade.buyer,
                price: trade.price,
                accepted_at: now,
            });

            Ok(())
        }

        /// Satıcı ticareti reddeder
        /// PendingSellerConfirm -> Refunded durumuna geçirir
        /// Alıcıya para iade edilir, ilan tekrar Active olur
        #[pallet::call_index(11)]
        #[pallet::weight(10_000)]
        pub fn reject_trade(
            origin: OriginFor<T>,
            trade_id: u64,
            reason_hash: Option<[u8; 32]>,
        ) -> DispatchResult {
            let seller = ensure_signed(origin)?;

            // Ticaret var mı?
            let mut trade = Trades::<T>::get(trade_id)
                .ok_or(Error::<T>::TradeNotFound)?;

            // Satıcı mı çağırıyor?
            ensure!(trade.seller == seller, Error::<T>::NotAuthorized);

            // PendingSellerConfirm durumunda mı?
            ensure!(
                trade.status == TradeStatus::PendingSellerConfirm,
                Error::<T>::NotPendingSellerConfirm
            );

            // Alıcıya ödemeyi iade et (fiyat + alıcı teminatı)
            // purchase'da reserve ile kilitlenmişti, unreserve ile geri ver
            let refund_amount = trade.price.saturating_add(trade.buyer_bond);
            T::Currency::unreserve(&trade.buyer, refund_amount);

            // Trade durumunu güncelle
            trade.status = TradeStatus::Refunded;
            Trades::<T>::insert(trade_id, trade.clone());

            // İlanı tekrar aktif yap
            if let Some(mut listing) = Listings::<T>::get(trade.listing_id) {
                listing.status = ListingStatus::Active;
                Listings::<T>::insert(trade.listing_id, listing);
            }

            Self::deposit_event(Event::TradeRejected {
                trade_id,
                seller,
                buyer: trade.buyer,
                reason_hash,
            });

            Ok(())
        }

        /// Teslimatı onayla - alıcı çağırır, satıcıya ödeme yapılır
        /// 
        /// - `delivery_attestation_hash`: Teslimat anında cihazdan alınan attestation hash'i (opsiyonel)
        ///   Buluşmada cihazın tekrar doğrulandığını kanıtlar (tam veri off-chain)
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn confirm_delivery(
            origin: OriginFor<T>,
            trade_id: u64,
            delivery_attestation_hash: Option<[u8; 32]>,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            // Trade'i bul
            let mut trade = Trades::<T>::get(trade_id)
                .ok_or(Error::<T>::TradeNotFound)?;

            // Sadece alıcı onaylayabilir
            ensure!(trade.buyer == caller, Error::<T>::NotAuthorized);

            // Escrow durumunda mı?
            ensure!(trade.status == TradeStatus::Escrow, Error::<T>::InvalidStatus);

            // Teslimat attestation hash ekle (varsa)
            trade.delivery_attestation_hash = delivery_attestation_hash;

            // Tanılama raporu varsa, report_hash'i de final_hash'e dahil et
            let diag_report_hash = if let Some(report) = <DiagnosticReports<T>>::get(trade_id) {
                Some(report.report_hash)
            } else {
                None
            };

            // Final hash hesapla (contract_hash + delivery_attestation + diagnostic_report)
            let final_hash = Self::compute_final_hash_with_diagnostics(
                trade.contract_hash,
                delivery_attestation_hash,
                diag_report_hash,
            );
            trade.final_hash = Some(final_hash);

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

            // Eğer delivery attestation hash eklendiyse, event yayınla
            if let Some(attestation_hash) = delivery_attestation_hash {
                Self::deposit_event(Event::DeliveryAttestationAdded {
                    trade_id,
                    attestation_hash,
                });
            }

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

        /// Anlaşmazlıkta koşul kanıtı sun (Merkle proof ile)
        /// 
        /// Alıcı veya satıcı, anlaşılan koşulları Merkle proof ile kanıtlayabilir.
        /// - `condition_hash`: Koşulun hash'i (key:value formatında)
        /// - `merkle_proof`: Sibling hash'ler (sabit boyutlu dizi)
        /// - `proof_len`: Proof'taki geçerli eleman sayısı
        /// - `index`: Yaprak index'i
        #[pallet::call_index(8)]
        #[pallet::weight(10_000)]
        pub fn submit_condition_proof(
            origin: OriginFor<T>,
            trade_id: u64,
            condition_hash: [u8; 32],
            merkle_proof: [[u8; 32]; 16], // Sabit boyut (MAX_MERKLE_PROOF_DEPTH)
            proof_len: u32,
            index: u32,
        ) -> DispatchResult {
            let submitter = ensure_signed(origin)?;

            // Trade'i bul
            let trade = Trades::<T>::get(trade_id)
                .ok_or(Error::<T>::TradeNotFound)?;

            // Alıcı veya satıcı mı?
            ensure!(
                trade.buyer == submitter || trade.seller == submitter,
                Error::<T>::NotAuthorized
            );

            // Sadece anlaşmazlık durumunda proof sunulabilir
            ensure!(trade.status == TradeStatus::Disputed, Error::<T>::InvalidStatus);

            // Proof uzunluğu geçerli mi?
            ensure!(proof_len <= MAX_MERKLE_PROOF_DEPTH, Error::<T>::MerkleProofTooDeep);

            // İlanı al (conditions_root için)
            let listing = Listings::<T>::get(trade.listing_id)
                .ok_or(Error::<T>::ListingNotFound)?;

            // Merkle proof doğrula (hash zaten hesaplanmış)
            let proof_slice = &merkle_proof[..(proof_len as usize)];
            let verified = Self::verify_merkle_proof_hashed(
                listing.conditions_root,
                condition_hash,
                proof_slice,
                index,
            );

            ensure!(verified, Error::<T>::InvalidMerkleProof);

            // Kanıtı kaydet
            let proof_index = <ProofCount<T>>::get(trade_id);
            let current_block = frame_system::Pallet::<T>::block_number();
            
            <SubmittedProofsSimple<T>>::insert(
                trade_id,
                proof_index,
                (condition_hash, submitter.clone(), current_block),
            );
            <ProofCount<T>>::insert(trade_id, proof_index + 1);

            // Event yayınla
            Self::deposit_event(Event::ConditionProofSubmitted {
                trade_id,
                submitter: submitter.clone(),
                condition_hash,
            });

            Self::deposit_event(Event::MerkleProofVerified {
                trade_id,
                condition_hash,
                verified,
            });

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

        /// Tanılama raporu sun (test sonuçlarını blockchain'e yaz)
        ///
        /// Satıcı veya alıcı çağırabilir. Her test sonucu ayrı ayrı zincire kaydedilir.
        /// Anlaşmazlıkta bu veriler kanıt olarak kullanılır.
        ///
        /// Parametreler:
        /// - `trade_id`: Ticaret ID'si
        /// - `device_model_hash`: blake2(cihaz_model_adı)
        /// - `device_manufacturer_hash`: blake2(üretici)
        /// - `os_hash`: blake2("Android 16" veya "iOS 18.2")
        /// - `test_ids`: Her testin id hash'i [blake2("screen_colors"), blake2("touch_grid"), ...]
        /// - `test_results`: Her testin sonucu (0=Failed, 1=Passed, 2=Skipped)
        /// - `test_details`: Her testin detay hash'i (açıklama hash'i)
        /// - `test_count`: Toplam test sayısı
        /// - `report_hash`: Tüm rapor verisinin birleşik hash'i
        #[pallet::call_index(9)]
        #[pallet::weight(50_000)]
        pub fn submit_diagnostics(
            origin: OriginFor<T>,
            trade_id: u64,
            device_model_hash: [u8; 32],
            device_manufacturer_hash: [u8; 32],
            os_hash: [u8; 32],
            test_ids: sp_runtime::BoundedVec<[u8; 32], frame_support::traits::ConstU32<16>>,
            test_results: sp_runtime::BoundedVec<u8, frame_support::traits::ConstU32<16>>,
            test_details: sp_runtime::BoundedVec<[u8; 32], frame_support::traits::ConstU32<16>>,
            report_hash: [u8; 32],
        ) -> DispatchResult {
            let submitter = ensure_signed(origin)?;

            // Trade'i bul
            let trade = Trades::<T>::get(trade_id)
                .ok_or(Error::<T>::TradeNotFound)?;

            // Sadece alıcı veya satıcı gönderebilir
            ensure!(
                trade.buyer == submitter || trade.seller == submitter,
                Error::<T>::NotAuthorized
            );

            // Escrow durumunda mı?
            ensure!(trade.status == TradeStatus::Escrow, Error::<T>::InvalidStatus);

            // Zaten tanılama sunulmuş mu?
            ensure!(!<HasDiagnostics<T>>::get(trade_id), Error::<T>::DiagnosticsAlreadySubmitted);

            // Test sayıları tutarlı mı?
            let count = test_ids.len() as u32;
            ensure!(count <= MAX_DIAGNOSTIC_TESTS, Error::<T>::TooManyDiagnosticTests);
            ensure!(test_results.len() as u32 == count, Error::<T>::TooManyDiagnosticTests);
            ensure!(test_details.len() as u32 == count, Error::<T>::TooManyDiagnosticTests);

            // Test sayılarını hesapla
            let mut passed_count: u32 = 0;
            let mut failed_count: u32 = 0;

            // Her testi ayrı ayrı kaydet
            for i in 0..count {
                let result = match test_results[i as usize] {
                    1 => { passed_count += 1; DiagTestResult::Passed },
                    0 => { failed_count += 1; DiagTestResult::Failed },
                    _ => DiagTestResult::Skipped,
                };

                let entry = DiagTestEntry {
                    test_id_hash: test_ids[i as usize],
                    result: result.clone(),
                    detail_hash: test_details[i as usize],
                };

                <DiagnosticTests<T>>::insert(trade_id, i, entry);

                // Her test için event yayınla
                Self::deposit_event(Event::DiagnosticTestRecorded {
                    trade_id,
                    test_index: i,
                    test_id_hash: test_ids[i as usize],
                    result,
                });
            }

            // Skor hesapla
            let score = if count > 0 {
                ((passed_count as u64 * 100) / count as u64) as u8
            } else {
                0u8
            };

            // Rapor oluştur ve kaydet
            let report = DiagnosticReport::<T> {
                submitter: submitter.clone(),
                device_model_hash,
                device_manufacturer_hash,
                os_hash,
                test_count: count,
                passed_count,
                failed_count,
                score,
                report_hash,
                submitted_at: frame_system::Pallet::<T>::block_number(),
            };

            <DiagnosticReports<T>>::insert(trade_id, report);
            <HasDiagnostics<T>>::insert(trade_id, true);

            // Ana event
            Self::deposit_event(Event::DiagnosticReportSubmitted {
                trade_id,
                submitter,
                device_model_hash,
                score,
                passed_count,
                failed_count,
                report_hash,
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
