# 📱 KOD Chain - Ticaret Sistemi

> **4'lü İmza ile Güvenli Cihaz Ticareti**

---

## 🎯 Vizyon

İki taraf birbirini tanımadan, güvenilir üçüncü parti olmadan, **cihazın kendisinin şahitliğinde** güvenli ticaret.

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│   "Satıcı, Alıcı, Cihaz ve Blockchain birlikte şahit olur"     │
│                                                                 │
│   • Manipülasyona karşı dayanıklı                               │
│   • Anlaşmazlıkta tam şeffaflık                                 │
│   • Herkes ne imzaladığını biliyor                              │
│   • Yalan söyleyen teminatını kaybediyor                        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🔐 4'lü İmza Sistemi

### Katılımcılar

| Taraf | Rolü | İmzası |
|-------|------|--------|
| **Satıcı** | Cihazı satıyor | "Bu özelliklerde satıyorum" |
| **Cihaz** | Kendini tanımlıyor | "Bu veriler benden çıktı" |
| **Alıcı** | Satın alıyor | "Kontrol ettim, kabul ediyorum" |
| **Madenciler** | Bloğa yazıyor | "Doğruladık, kaydettik" |

### Görsel

```
  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
  │  SATICI  │  │  CİHAZ   │  │  ALICI   │  │ MADENCİ  │
  │  (Alice) │  │ (iPhone) │  │  (Bob)   │  │ (Miners) │
  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘
       │             │             │             │
       ▼             ▼             ▼             ▼
   "Satıyorum"   "Ben bu       "Aldım,       "Bloğa
   "Özellikler    cihazım"     kontrol       yazdık"
    şunlar"      "Veriler      ettim"       "Doğruladık"
                  benden"      "Kabul"
       │             │             │             │
       └──────────┬──┴─────────────┴─────────────┘
                  │
                  ▼
         ┌───────────────────┐
         │    BLOCKCHAIN     │
         │                   │
         │  seller_sig ✅    │
         │  device_sig ✅    │
         │  buyer_sig  ✅    │
         │  block_hash ✅    │
         │                   │
         │  HERKES ŞAHİT!    │
         └───────────────────┘
```

---

## 📋 Ticaret Akışı

### Aşama 1: İlan Oluşturma

```
Alice telefon satmak istiyor
        │
        ▼
┌─────────────────────────────────────────────┐
│  📱 Mobil Uygulama                          │
│                                             │
│  Otomatik Veriler (Cihazdan):               │
│  ├── Model: iPhone 17 Pro Max               │
│  ├── Depolama: 256GB                        │
│  ├── Batarya Sağlığı: 92%                   │
│  ├── GPS: Çalışıyor                         │
│  ├── Mikrofon: Çalışıyor                    │
│  └── Hoparlör: Çalışıyor                    │
│                                             │
│  Manuel Bilgiler (Satıcıdan):               │
│  ├── iCloud: Hesaptan çıkıldı               │
│  ├── Find My iPhone: Kapalı                 │
│  ├── Şifre: Kaldırıldı                      │
│  └── Fabrika Ayarları: Yapıldı              │
│                                             │
└─────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────┐
│  🔐 İmzalama                                │
│                                             │
│  1. Cihaz İmzası:                           │
│     → Secure Enclave/Keystore'da anahtar    │
│     → "Bu veriler iPhone18,1 cihazından"    │
│     → device_signature                      │
│                                             │
│  2. Satıcı İmzası:                          │
│     → Alice'in blockchain cüzdanı           │
│     → "Bu koşullarla satıyorum"             │
│     → seller_signature                      │
│                                             │
└─────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────┐
│  ⛓️ Blockchain'e Yazılan                    │
│                                             │
│  Listing {                                  │
│    seller: Alice,                           │
│    price: 1000 KOD,                         │
│    bond: 150 KOD (teminat),                 │
│    conditions_root: 0xabc123...,            │
│    device_attestation: {                    │
│      model: "iPhone18,1",                   │
│      data_hash: 0xdef456...,                │
│      device_signature: 0x789...,            │
│      device_public_key: 0xaaa...            │
│    },                                       │
│    seller_signature: 0xbbb...,              │
│    status: Active                           │
│  }                                          │
│                                             │
└─────────────────────────────────────────────┘
```

### Aşama 2: Satın Alma

```
Bob ilanı görüyor ve beğeniyor
        │
        ▼
┌─────────────────────────────────────────────┐
│  🛒 Satın Alma Teklifi                      │
│                                             │
│  Bob:                                       │
│  ├── Fiyat teklifi: 1000 KOD                │
│  ├── Teminat: 150 KOD                       │
│  └── Koşulları kabul ediyor                 │
│                                             │
│  Bob'un İmzası:                             │
│  → "Bu koşulları kabul ediyorum"            │
│  → buyer_signature                          │
│                                             │
└─────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────┐
│  ✅ Alice Kabul Ediyor                      │
│                                             │
│  Alice'in Onay İmzası:                      │
│  → "Bob'un teklifini kabul ediyorum"        │
│  → seller_acceptance_signature              │
│                                             │
└─────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────┐
│  ⛓️ Blockchain'e Yazılan                    │
│                                             │
│  Trade {                                    │
│    listing_id: 0,                           │
│    buyer: Bob,                              │
│    seller: Alice,                           │
│    price: 1000 KOD,                         │
│    buyer_bond: 150 KOD,                     │
│    seller_bond: 150 KOD,                    │
│    contract_hash: 0xccc...,                 │
│    buyer_signature: 0xddd...,               │
│    seller_acceptance: 0xeee...,             │
│    status: Escrow                           │
│  }                                          │
│                                             │
│  💰 Escrow'da Kilitli:                      │
│  ├── Bob'un ödemesi: 1000 KOD               │
│  ├── Bob'un teminatı: 150 KOD               │
│  └── Alice'in teminatı: 150 KOD             │
│                                             │
└─────────────────────────────────────────────┘
```

### Aşama 3: Buluşma ve Teslimat

```
Alice ve Bob buluşuyor (yüz yüze)
        │
        ▼
┌─────────────────────────────────────────────┐
│  📱 Cihaz Kontrolü                          │
│                                             │
│  Bob, Alice'in telefonunu alıyor            │
│                                             │
│  Otomatik Testler:                          │
│  ├── Batarya: 92% ✅                        │
│  ├── GPS: Çalışıyor ✅                      │
│  ├── Mikrofon: Çalışıyor ✅                 │
│  └── Hoparlör: Çalışıyor ✅                 │
│                                             │
│  Manuel Kontroller:                         │
│  ├── iCloud: Çıkış yapılmış ✅              │
│  ├── Find My: Kapalı ✅                     │
│  ├── Şifre: Yok ✅                          │
│  └── Ekran: Çiziksiz ✅                     │
│                                             │
└─────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────┐
│  🔐 Teslimat İmzaları                       │
│                                             │
│  1. Cihaz İmzası (Yeni):                    │
│     → Aynı cihaz, güncel veriler            │
│     → device_delivery_signature             │
│                                             │
│  2. Alıcı İmzası:                           │
│     → "Kontrol ettim, her şey OK"           │
│     → buyer_confirmation_signature          │
│                                             │
│  3. Satıcı İmzası:                          │
│     → "Teslim ettim"                        │
│     → seller_delivery_signature             │
│                                             │
└─────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────┐
│  ⛓️ Blockchain'e Yazılan                    │
│                                             │
│  TradeCompletion {                          │
│    trade_id: 0,                             │
│    device_attestation: {                    │
│      data_hash: 0xfff...,                   │
│      device_signature: 0x111...,            │
│    },                                       │
│    buyer_confirmation: 0x222...,            │
│    seller_delivery: 0x333...,               │
│    final_hash: 0x444...,                    │
│    status: Completed                        │
│  }                                          │
│                                             │
│  💰 Ödeme Serbest:                          │
│  ├── Alice'e: 1000 KOD (ödeme)              │
│  ├── Alice'e: 150 KOD (teminat iade)        │
│  └── Bob'a: 150 KOD (teminat iade)          │
│                                             │
└─────────────────────────────────────────────┘
```

---

## 🏦 TL Ödeme Akışı

Fiyat **TL**, ödeme **banka havalesi** ile yapılır; blockchain'de sadece **KOD teminatı** (%10) kilitlenir. IBAN'lar off-chain paylaşılır, on-chain sadece **IBAN hash** saklanır.

### Akış Özeti

```
Satıcı: create_listing(tl_price=15000000, seller_iban_hash)  → Bond = TL değerin %10'u (KOD)
Alıcı:  purchase(listing_id, buyer_bond, buyer_iban_hash)    → Sadece bond kilitlenir
Satıcı: accept_trade(...)                                     → Durum: AwaitingPayment
Alıcı:  Banka havalesi (off-chain) → mark_payment_sent()     → Durum: PaymentSent
Satıcı: confirm_tl_payment()                                  → Tamamlandı, teminatlar iade
```

### On-Chain Veriler

| Veri | Açıklama |
|------|----------|
| `tl_price` | TL fiyat (kuruş; 15000000 = 150.000 TL) |
| `seller_iban_hash` | blake2(IBAN) – ilan/trade'de |
| `buyer_iban_hash` | blake2(IBAN) – trade'de |
| `KodTlRate` | KOD/TL kuru (kuruş; 100 = 1 KOD = 1 TL), sudo ile güncellenir |

### Yeni Trade Durumları

| Durum | Açıklama |
|-------|----------|
| `AwaitingPayment` | Satıcı kabul etti; alıcı TL havale yapacak |
| `PaymentSent` | Alıcı havaleyi yaptığını bildirdi; satıcı onayı bekleniyor |

Anlaşmazlıkta alıcı banka dekontu ile kanıt sunar; hakem `resolve_dispute` ile karar verir.

---

## ⚖️ Anlaşmazlık Durumu

### Senaryo: Batarya Sorunu

```
Bob: "Batarya %50, %92 değildi!"

┌─────────────────────────────────────────────────────────────┐
│                     HAKEM İNCELEMESİ                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  📜 İLAN AŞAMASI (Blok #1000)                              │
│  ─────────────────────────────                              │
│                                                             │
│  Satıcı İmzası: Alice ✅                                   │
│    → "iPhone 17 Pro Max 256GB satıyorum"                   │
│    → "Batarya %92"                                         │
│                                                             │
│  Cihaz İmzası: iPhone18,1 ✅                               │
│    → Model: iPhone 17 Pro Max                              │
│    → Batarya: 92%                                          │
│    → Zaman: 2026-02-04 10:00                               │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  🤝 ANLAŞMA AŞAMASI (Blok #1050)                           │
│  ───────────────────────────────                            │
│                                                             │
│  Alıcı İmzası: Bob ✅                                      │
│    → "Bu koşulları kabul ediyorum"                         │
│    → "Batarya %92 olacak"                                  │
│                                                             │
│  Satıcı Onayı: Alice ✅                                    │
│    → "Anlaşmayı onaylıyorum"                               │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  📦 TESLİMAT AŞAMASI (Blok #1100)                          │
│  ─────────────────────────────────                          │
│                                                             │
│  Cihaz İmzası: iPhone18,1 ✅                               │
│    → Batarya: 50% ⚠️ FARKLI!                               │
│    → Zaman: 2026-02-04 15:00                               │
│                                                             │
│  Alıcı: openDispute() ✅                                   │
│    → "Batarya söylenenden farklı!"                         │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  🔍 MERKLE PROOF DOĞRULAMA                                 │
│  ──────────────────────────                                 │
│                                                             │
│  Orijinal koşul: "battery: 92%"                            │
│  Merkle proof: [0x123..., 0x456...]                        │
│  Root doğrulama: ✅ Eşleşiyor                              │
│                                                             │
│  → Bu koşul gerçekten sözleşmedeydi                        │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ⚖️ KARAR                                                   │
│  ────────                                                   │
│                                                             │
│  İlanda cihaz imzası: batarya = 92%                        │
│  Teslimatta cihaz imzası: batarya = 50%                    │
│                                                             │
│  OLASI DURUMLAR:                                           │
│  1. Farklı cihaz teslim edildi                             │
│  2. Batarya bu sürede bozuldu (satıcı sorumlu)             │
│                                                             │
│  → ALICI (BOB) HAKLI!                                      │
│                                                             │
│  SONUÇ:                                                     │
│  ├── Bob'a: 1000 KOD (ödeme iade)                          │
│  ├── Bob'a: 150 KOD (kendi teminatı)                       │
│  ├── Bob'a: 150 KOD (Alice'in teminatı - ceza)             │
│  └── Alice: 0 KOD (teminat kaybetti)                       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 📱 Cihaz İmzası Teknik Detaylar

### iOS (Secure Enclave)

```swift
// Secure Enclave'de anahtar oluştur
let access = SecAccessControlCreateWithFlags(
    nil,
    kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
    .privateKeyUsage,
    nil
)

let attributes: [String: Any] = [
    kSecAttrKeyType: kSecAttrKeyTypeECSECPrimeRandom,
    kSecAttrKeySizeInBits: 256,
    kSecAttrTokenID: kSecAttrTokenIDSecureEnclave,
    kSecPrivateKeyAttrs: [
        kSecAttrIsPermanent: true,
        kSecAttrAccessControl: access
    ]
]

let privateKey = SecKeyCreateRandomKey(attributes, nil)

// Veriyi imzala
let dataToSign = """
{
    "model": "iPhone18,1",
    "battery": "92%",
    "timestamp": "2026-02-04T10:00:00Z"
}
""".data(using: .utf8)!

let signature = SecKeyCreateSignature(
    privateKey,
    .ecdsaSignatureMessageX962SHA256,
    dataToSign as CFData,
    nil
)
```

### Android (Keystore with StrongBox)

```kotlin
// StrongBox'ta anahtar oluştur
val keyGenerator = KeyPairGenerator.getInstance(
    KeyProperties.KEY_ALGORITHM_EC,
    "AndroidKeyStore"
)

keyGenerator.initialize(
    KeyGenParameterSpec.Builder("device_key", PURPOSE_SIGN)
        .setDigests(KeyProperties.DIGEST_SHA256)
        .setIsStrongBoxBacked(true)  // Hardware güvenlik
        .build()
)

val keyPair = keyGenerator.generateKeyPair()

// Veriyi imzala
val signature = Signature.getInstance("SHA256withECDSA")
signature.initSign(keyPair.private)
signature.update(dataToSign)
val deviceSignature = signature.sign()
```

---

## 📊 Veri Yapıları

### Kontrat JSON (Off-chain, IPFS'te)

```json
{
  "version": "1.0",
  "type": "smartphone",
  
  "device": {
    "brand": "Apple",
    "model": "iPhone 17 Pro Max",
    "model_id": "iPhone18,1",
    "storage": "256GB",
    "color": "Natural Titanium"
  },
  
  "conditions": {
    "automatic": {
      "battery_health": {"value": "92%", "min_acceptable": "80%"},
      "gps": {"value": "working"},
      "microphone": {"value": "working"},
      "speaker": {"value": "working"},
      "wifi": {"value": "working"},
      "bluetooth": {"value": "working"}
    },
    "manual": {
      "icloud_status": {"value": "signed_out"},
      "find_my_iphone": {"value": "disabled"},
      "passcode": {"value": "removed"},
      "factory_reset": {"value": "completed"},
      "screen_condition": {"value": "no_scratch"},
      "body_condition": {"value": "minor_wear"}
    }
  },
  
  "accessories": {
    "original_box": true,
    "charger": true,
    "cable": true,
    "manual": false
  },
  
  "seller_notes": "6 ay kullanıldı, hiç düşürülmedi."
}
```

### Blockchain'deki Veri

```rust
// Listing yapısı
struct Listing<T: Config> {
    seller: AccountId,
    price: Balance,
    seller_bond: Balance,
    
    // Merkle sistemi
    conditions_root: [u8; 32],
    ipfs_cid: Option<Vec<u8>>,
    
    // Cihaz attestation
    device_attestation: DeviceAttestation,
    
    // İmzalar
    seller_signature: Signature,
    
    status: ListingStatus,
    created_at: BlockNumber,
}

// Cihaz attestation
struct DeviceAttestation {
    model_id: Vec<u8>,          // "iPhone18,1"
    model_name: Vec<u8>,        // "iPhone 17 Pro Max"
    storage: Vec<u8>,           // "256GB"
    data_hash: [u8; 32],        // Tüm verilerin hash'i
    device_signature: Vec<u8>,  // Cihaz imzası
    device_public_key: Vec<u8>, // Cihaz public key
    timestamp: u64,
}

// Trade yapısı
struct Trade<T: Config> {
    listing_id: u64,
    buyer: AccountId,
    seller: AccountId,
    price: Balance,
    buyer_bond: Balance,
    seller_bond: Balance,
    
    // Anlaşma imzaları
    contract_hash: [u8; 32],
    buyer_signature: Signature,
    seller_acceptance: Signature,
    
    // Teslimat imzaları (sonra eklenir)
    delivery_device_attestation: Option<DeviceAttestation>,
    buyer_confirmation: Option<Signature>,
    seller_delivery: Option<Signature>,
    
    final_hash: Option<[u8; 32]>,
    status: TradeStatus,
    created_at: BlockNumber,
}
```

---

## 🔒 Güvenlik Modeli

### Teminat Sistemi

```
TEMİNAT ORANLARI (Örnek)
════════════════════════

Ürün Fiyatı: 1000 KOD

Satıcı Teminatı: %15 = 150 KOD
Alıcı Teminatı:  %15 = 150 KOD

TOPLAM KİLİTLİ: 1300 KOD
├── Alıcı ödemesi: 1000 KOD
├── Alıcı teminatı: 150 KOD
└── Satıcı teminatı: 150 KOD


SONUÇLAR:
─────────

✅ Her şey OK:
├── Satıcıya: 1000 KOD (ödeme) + 150 KOD (teminat)
└── Alıcıya: 150 KOD (teminat)

❌ Satıcı suçlu:
├── Alıcıya: 1000 KOD (iade) + 150 KOD (kendi) + 150 KOD (satıcıdan)
└── Satıcıya: 0 KOD

❌ Alıcı suçlu (yalan söyledi):
├── Satıcıya: 1000 KOD + 150 KOD (kendi) + 150 KOD (alıcıdan)
└── Alıcıya: 0 KOD
```

### Neden 4'lü İmza?

```
SALDIRI SENARYOLARI VE KORUMALARI
═════════════════════════════════

1. Satıcı yalan söylerse:
   → Cihaz imzası farklı veri gösterir
   → Kanıt: device_signature
   
2. Alıcı yalan söylerse:
   → Kendi imzasıyla kabul etmişti
   → Kanıt: buyer_confirmation
   
3. Sahte cihaz:
   → Cihaz imzası farklı model gösterir
   → Public key eşleşmez
   → Kanıt: device_public_key
   
4. Manipülasyon:
   → Tüm imzalar blockchain'de
   → Değiştirilemez (immutable)
   → Kanıt: block_hash
```

---

## 🛣️ Yol Haritası

### Aşama 1: Temel Sistem
- [x] Trade Pallet (mevcut)
- [x] Merkle proof sistemi (conditions_root on-chain)
- [x] Satıcı kabul/red mekanizması (PendingSellerConfirm)
- [x] Cihaz tanılama on-chain (submit_diagnostics)
- [ ] Çoklu imza yapısı (gelecek)

### Aşama 2: Cihaz Entegrasyonu
- [x] iOS mobil uygulama
- [x] Android mobil uygulama
- [x] Secure Enclave/Keystore entegrasyonu (device attestation)
- [x] Otomatik veri toplama (expo-device, expo-file-system)
- [x] PhoneDoctor tarzı 12 donanım/yazılım testi
- [x] Otomatik QR oluşturma (testler bitince)
- [x] QR tarama ve blockchain'de doğrulama

### Aşama 3: Sözleşme Gizliliği ✅
- [x] **NaCl Box şifreleme** (x25519-xsalsa20-poly1305)
- [x] **EncryptedContracts** on-chain storage (max 8KB)
- [x] **ContractEncryptionKeys** per-party key wrapping
- [x] **Explorer sayfası** (kod.services/explorer.html)
- [x] **12 kelime ile tarayıcıda deşifreleme** (client-side)

### Aşama 4: Varlık Sicili (v7.0.0) ✅
- [x] **Asset Registry** — ürün sahipliği on-chain takip
- [x] `Asset` struct: current_owner, original_owner, asset_hash, transfer_count
- [x] `OwnershipRecord`: from, to, trade_id, transfer_type (Sale/TlSale/DisputeResolution)
- [x] `create_listing` ile otomatik varlık kaydı
- [x] Ticaret tamamlanınca sahiplik otomatik devri
- [x] İkinci el desteği: aynı ürün mevcut asset_id ile yeniden ilanlanabilir
- [x] Explorer'dan sorgulanabilir: `assets`, `assetOwnershipHistory`, `ownerAssets`

### Aşama 5: Gelişmiş Özellikler
- [ ] Kargo kontratı (time-lock)
- [ ] Hakem sistemi (çoklu hakem)
- [ ] Puan/değerlendirme

---

## 📱 Mobil Uygulama Entegrasyonu

### Teknoloji Stack

```
┌────────────────────────────────────────────┐
│         KOD Mobile (kod-mobile/)           │
├────────────────────────────────────────────┤
│  • React Native 0.76.5                     │
│  • Expo SDK 52                             │
│  • @polkadot/api (WebSocket RPC)           │
│  • Supabase (PostgreSQL backend)           │
│  • Zustand (state management)              │
│  • expo-device, expo-file-system           │
└────────────────────────────────────────────┘
```

### Gerçek Dünya Testi (2026-02-07/09)

**Test Edilen:**
- ✅ Cüzdan oluşturma (`//Alice`, `//Bob` seed phrases)
- ✅ WebSocket bağlantısı (`ws://192.168.1.135:9944`)
- ✅ Otomatik cihaz tespiti (Xiaomi 17 Pro Max 512GB 16GB RAM)
- ✅ İlan oluşturma (`createListing` extrinsic, 6 argüman)
- ✅ Satın alma (`purchase` extrinsic)
- ✅ Satıcı kabul (`accept_trade` extrinsic)
- ✅ Cihaz tanılama testleri (12 test, otomatik QR)
- ✅ Tanılama raporu blockchain'e yazma (`submit_diagnostics`)
- ✅ Teslimat onayı (`confirmDelivery`)
- ✅ Blockchain bakiye sorgulama
- ✅ Gerçek zamanlı ilan görüntüleme
- ✅ Crypto polyfills (`react-native-get-random-values`)

**Test Cihazlar:**
- Telefon 1: Xiaomi 17 Pro Max (Android, Satıcı - Alice)
- Telefon 2: iPhone 12 Pro (iOS, Alıcı - Bob)

**Tam Ticaret Akışı Test Edildi:**
```
Alice ilan oluşturur → Bob satın alır → Alice kabul eder (BC'ye yazılır) 
→ Alice cihaz testlerini yapar → Otomatik QR oluşur
→ Bob QR tarar → Sonuçlar blockchain'de doğrulanır → Ödeme serbest
```

### Teknik Zorluklar ve Çözümler

#### 1. Crypto Polyfills

**Sorun:** React Native (Hermes) `crypto.getRandomValues` API'sini desteklemiyor.

**Çözüm:**
```typescript
// src/polyfills.ts
import 'react-native-get-random-values';
globalThis.crypto = global.crypto;

// metro.config.js (module override)
config.resolver.resolveRequest = (context, moduleName, platform) => {
  if (moduleName === '@noble/hashes/crypto') {
    return { filePath: './src/noble-crypto-shim.js' };
  }
};

// src/noble-crypto-shim.js (Proxy ile dinamik erişim)
export const crypto = new Proxy({}, {
  get(target, prop) {
    return globalThis.crypto?.[prop];
  }
});
```

#### 2. Runtime Metadata Uyumu

**Sorun:** `Extrinsic trade.createListing expects 4 arguments got 6`

**Çözüm:**
```typescript
// Runtime metadata sorgula
const createListingMeta = api.tx.trade.createListing.meta;
console.log('Arguments:', createListingMeta.args.length); // 4

// Doğru imza (4 argüman)
api.tx.trade.createListing(
  price,          // Balance
  bond,           // Balance
  dataHash,       // [u8; 32] (Merkle root)
  acceptsExternal // bool
);
```

#### 3. Minimum Bond Gereksinimi

**Sorun:** `trade.InsufficientBond: Yetersiz teminat`

**Sebep:** Runtime'da `MinBond = 10 KOD` tanımlı:
```rust
// runtime/src/configs/mod.rs
parameter_types! {
    pub const MinBond: Balance = 10 * UNITS; // 10 KOD
}
```

**Çözüm:** Minimum 100 KOD fiyat kullan (bond %10 = 10 KOD)

#### 4. Bakiye Senkronizasyonu

**Sorun:** Mobil uygulamada bakiye `0 KOD` gösteriyordu.

**Çözüm:**
```typescript
// src/store/wallet.ts
refreshBalance: async () => {
  const chainService = (await import('../services/chain')).default;
  const balanceData = await chainService.getBalance(wallet.address);
  set({ balance: balanceData.free });
}

// app/(tabs)/profile.tsx
useEffect(() => {
  if (wallet && chainInfo.connected) {
    refreshBalance();
    const interval = setInterval(refreshBalance, 10000); // Her 10s
    return () => clearInterval(interval);
  }
}, [wallet, chainInfo.connected]);
```

### Ekran Görüntüleri

#### Sat Ekranı (Otomatik Tespit)
```
┌──────────────────────────────┐
│  🏷️  Bu Cihazı Sat           │
├──────────────────────────────┤
│  Cihaz Bilgileri             │
│  ├─ Xiaomi 17 Pro Max        │
│  ├─ 512 GB Depolama          │
│  └─ 16 GB RAM                │
│                              │
│  Fiyat: [100] KOD            │
│  Teminat (Oto): 10 KOD       │
│                              │
│  [İlan Oluştur]              │
└──────────────────────────────┘
```

#### Keşfet Ekranı (Blockchain İlanları)
```
┌──────────────────────────────┐
│  🔍 Keşfet                    │
├──────────────────────────────┤
│  Blockchain İlanları (1)     │
│                              │
│  ┌────────────────────────┐  │
│  │ #0 - Aktif             │  │
│  │ 100 KOD                │  │
│  │ Satıcı: 5GrwvaEF...    │  │
│  │ Teminat: 10 KOD        │  │
│  └────────────────────────┘  │
└──────────────────────────────┘
```

#### Profil Ekranı (Bakiye)
```
┌──────────────────────────────┐
│  👤 Profil                    │
├──────────────────────────────┤
│  Adres: 5GrwvaEF5zXb...      │
│  Bakiye: 59,999,999.9999 KOD │
│                              │
│  Ağ Ayarları                 │
│  ├─ Node: 192.168.1.93:9944  │
│  └─ Durum: ✅ Bağlı          │
└──────────────────────────────┘
```

### Dokümantasyon

Detaylı teknik dokümantasyon:
- [kod-mobile/README.md](../../kod-mobile/README.md) - Kurulum ve kullanım
- [kod-mobile/docs/TECHNICAL.md](../../kod-mobile/docs/TECHNICAL.md) - Mimari ve sorun giderme

---

## 🔐 Şifreli Sözleşme Sistemi (v4.0.0)

### Neden Şifreleme?

Trade sözleşmesi blockchain'e yazıldığında herkes görebilir. Tarafların gizliliğini korumak için sözleşme içeriği **NaCl Box** ile şifrelenir.

### Nasıl Çalışır?

```
                    ŞİFRELEME AKIŞI
                    ════════════════

1. Her kullanıcı cüzdan oluştururken x25519 keypair türetir
   └── Seed phrase → miniSecret → nacl.box.keyPair

2. x25519 public key Supabase'e kaydedilir
   └── kodcoin_address kolonu

3. Satıcı ticareti kabul ederken:
   ┌─────────────────────────────────────────────┐
   │  a) Random simetrik anahtar üret (32 byte)  │
   │  b) Sözleşme JSON'u → nacl.secretbox ile    │
   │     simetrik anahtarla şifrele               │
   │  c) Simetrik anahtarı → nacl.box ile         │
   │     alıcının x25519 pubkey'i ile şifrele     │
   │  d) Simetrik anahtarı → nacl.box ile         │
   │     satıcının x25519 pubkey'i ile şifrele    │
   └─────────────────────────────────────────────┘

4. Blockchain'e yazılır:
   ├── EncryptedContracts[trade_id] = şifreli sözleşme
   ├── ContractEncryptionKeys[trade_id][buyer] = buyer wrapped key
   └── ContractEncryptionKeys[trade_id][seller] = seller wrapped key

5. Deşifreleme (Explorer veya mobil uygulama):
   ┌─────────────────────────────────────────────┐
   │  a) 12 kelime → miniSecret → x25519 secret  │
   │  b) Wrapped key → nacl.box.open → simetrik  │
   │  c) Şifreli içerik → nacl.secretbox.open    │
   │  d) JSON parse → okunabilir sözleşme         │
   └─────────────────────────────────────────────┘
```

### Wrapped Key Formatı

```
ephemeralPublicKey (32 byte) + nonce (24 byte) + encryptedSymKey (48 byte)
= toplam 104 byte
```

### Explorer (kod.services/explorer.html)

Web tabanlı sözleşme görüntüleyici:
- Node'a WebSocket ile bağlanır
- Trade ID ile sorgulama
- Zincir istatistikleri (toplam trade, hacim, ilan sayısı)
- 12 kelime ile client-side deşifreleme (sunucuya hiçbir şey gönderilmez)
- Vite + vanilla JS, tek HTML dosyası (inline JS/CSS)

### Güvenlik

| Özellik | Detay |
|---------|-------|
| Algoritma | NaCl Box (x25519-xsalsa20-poly1305) |
| Simetrik | NaCl SecretBox (xsalsa20-poly1305) |
| Key Exchange | Ephemeral Diffie-Hellman |
| Deşifreleme | Sadece client-side (tarayıcı/uygulama) |
| Sunucu | Hiçbir private key sunucuya gitmez |

---

## 📦 Varlık Sicili (Asset Registry — v7.0.0)

Her ürün blockchain'de benzersiz bir kimlikle takip edilir. Ticaret tamamlandığında sahiplik otomatik devredilir.

```
                     VARLIK SİCİLİ AKIŞI
                     ═══════════════════

1. İLAN OLUŞTUR
   Alice → create_listing(...)
   → AssetRegistered { asset_id: 0, owner: Alice, asset_hash: 0x... }
   → Asset { current_owner: Alice, original_owner: Alice, transfer_count: 0 }

2. TİCARET TAMAMLANDI
   Bob → confirm_delivery / confirm_tl_payment
   → OwnershipTransferred { asset_id: 0, from: Alice, to: Bob, transfer_type: Sale }
   → Asset { current_owner: Bob, original_owner: Alice, transfer_count: 1 }

3. İKİNCİ EL SATIŞ
   Bob → create_listing(...) (aynı ürün)
   → Mevcut asset_id: 0 kullanılır (asset_hash eşleşir)

4. Charlie alır → transfer_count: 2

   EXPLORER SORGUSU:
   ┌──────────────────────────────────────────┐
   │  assets(0) →                             │
   │    current_owner: Charlie                │
   │    original_owner: Alice                 │
   │    transfer_count: 2                     │
   │                                          │
   │  assetOwnershipHistory(0) →              │
   │    [0] Alice → Bob   (Trade #1, Sale)    │
   │    [1] Bob → Charlie (Trade #5, TlSale)  │
   │                                          │
   │  ownerAssets(Charlie) → [0]              │
   └──────────────────────────────────────────┘
```

### Transfer Tipleri

| Tip | Açıklama |
|-----|----------|
| `Sale` | Normal KOD ticareti tamamlandı |
| `TlSale` | TL ödemeli ticaret tamamlandı |
| `DisputeResolution` | Anlaşmazlık sonucu alıcıya verildi |

---

## 📝 Özet

```
KOD Chain Ticaret Sistemi
═════════════════════════

Kim imzalıyor?
├── Satıcı: "Satıyorum, koşullar şu"
├── Cihaz: "Veriler benden, modelim şu"
├── Alıcı: "Kabul ediyorum / Onaylıyorum"
└── Madenciler: "Bloğa yazdık"

Ne zaman?
├── İlan: Satıcı + Cihaz imzası
├── Anlaşma: Alıcı + Satıcı imzası
└── Teslimat: Cihaz + Alıcı + Satıcı imzası

Ticaret tamamlanınca?
├── Sahiplik otomatik devredilir (Asset Registry)
├── Tam geçmiş on-chain'de saklanır
└── İkinci el satışlarda ürün sicili korunur

Anlaşmazlıkta?
├── Tüm imzalar blockchain'de
├── Merkle proof ile koşul kanıtı
├── Cihaz imzaları karşılaştırılır
└── Hakem net kanıtla karar verir

Sonuç:
└── Yalan söyleyen teminatını kaybeder!
```

---

<div align="center">

**KOD Chain** - *Güvenilir Ticaretin Geleceği* 🔗

</div>

