# 🔗 KOD Chain

> **Güvenilir, Merkeziyetsiz Ticaret Platformu**

---

## 🎯 Vizyon

KOD Chain, **aracısız ve güvenilir ticaret** için tasarlanmış bir blockchain altyapısıdır.

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                  │
│   "İki taraf birbirini tanımadan, güvenle ticaret yapabilmeli"  │
│                                                                  │
│   • Manipülasyona karşı dayanıklı                               │
│   • Merkezi otoriteye bağımlı değil                             │
│   • Kurallar kod ile uygulanır                                  │
│   • Herkes kendi node'unu çalıştırabilir                        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Temel Prensipler

| Prensip | Açıklama |
|---------|----------|
| **Trustless** | Güven, insanlara değil koda dayanır |
| **Decentralized** | Tek bir kontrol noktası yok |
| **Sovereign** | Her kullanıcı kendi node'unu çalıştırır |
| **Fair Launch** | Ön madencilik yok, herkes eşit başlar |

---

## 🏗️ Mimari

```
                         KOD CHAIN MİMARİSİ
                         ═══════════════════

┌─────────────────────────────────────────────────────────────────┐
│                        UYGULAMA KATMANI                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Trade Web   │  │   Wallet     │  │   Explorer   │          │
│  │   (Next.js)  │  │    (CLI)     │  │  (Polkadot)  │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        RUNTİME (WASM)                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │    Trade     │  │ Block Reward │  │   Balances   │          │
│  │   Pallet     │  │    Pallet    │  │   Pallet     │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Timestamp   │  │     Sudo     │  │   System     │          │
│  │   Pallet     │  │    Pallet    │  │   Pallet     │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        NODE (Native)                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Mining     │  │   Network    │  │     RPC      │          │
│  │  (SHA3-256)  │  │  (libp2p)    │  │   (JSON)     │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│  ┌──────────────┐  ┌──────────────┐                            │
│  │   Storage    │  │ Auto-Wallet  │                            │
│  │  (RocksDB)   │  │  (sr25519)   │                            │
│  └──────────────┘  └──────────────┘                            │
└─────────────────────────────────────────────────────────────────┘
```

---

## 💰 Ekonomi

### Token Dağılımı

```
TOPLAM ARZ: 1,000,000,000 KOD (1 Milyar)
═══════════════════════════════════════

┌────────────────────────────────────────────────────────┐
│████████████████████████████████████████████████░░░░░░░│
│          Madencilik Havuzu (900M - %90)        │ Takım │
│                                                 │ 100M  │
│                                                 │ %10   │
└────────────────────────────────────────────────────────┘
```

| Havuz | Miktar | Oran | Açıklama |
|-------|--------|------|----------|
| **Madencilik** | 900,000,000 KOD | %90 | Blok ödülleri için ayrılmış |
| **Takım/Geliştirme** | 100,000,000 KOD | %10 | Genesis'te dağıtılmış |

### Blok Ödülleri ve Halving

```
Blok Süresi: 6 saniye
İlk Ödül: 50 KOD/blok
Halving: Her 10,500,000 blok (~2 yıl)

Dönem    Blok Aralığı           Ödül/Blok    Süre
────────────────────────────────────────────────────
Era 0    0 - 10,499,999          50 KOD       ~2 yıl
Era 1    10,500,000 - 20,999,999  25 KOD      ~2 yıl
...
Era 10+  105,000,000+             ~0 KOD       ∞
```

### KOD-Only Modu

```
Blok 21,000,000'den sonra (~4 yıl):
• Sadece KOD ile ticaret yapılabilir
• External ödeme (ETH, BTC, USDT) kabul eden ilanlar engellenir.
```

---

## 🏪 Trade Pallet - Ticaret Sistemi

### Nasıl Çalışır?

```
                         TİCARET AKIŞI
                         ═════════════

    SATICI                  ESCROW                    ALICI
    ══════                  ══════                    ═════

1️⃣ İlan Ver
    │
    │  💎 Teminat (Bond)
    └──────────────────────▶ 🔒 Kilitli
    
2️⃣                                              Satın Al
                                                     │
                             🔒 Kilitli ◀────────────┘
                             (Fiyat + Bond)

3️⃣ Ürün Gönder ──────────────────────────────▶ Ürün Al

4️⃣                                              Onayla
                                                     │
    💰 Ödeme ◀───────────── 🔓 Serbest ◀────────────┘
    💎 Bond iade                💎 Bond iade ────────▶

                         ═════════════════
                         ✅ TİCARET TAMAM!
```

### Anlaşmazlık Durumu

```
    SATICI                  HAKEM                     ALICI
    ══════                  ═════                     ═════

    Ürün gönderildi! ────────────────────────▶ Ürün bozuk!
                                │
                       Kanıtları incele
                                │
                       ┌────────┴────────┐
                       ▼                 ▼
                  Satıcı Haklı      Alıcı Haklı
                       │                 │
    💰 Ödeme ◀─────────┘                 └──────────▶ 💰 İade
    💎 Kendi bond                              💎 Kendi bond
    💎 Alıcı bond                              💎 Satıcı bond
```

### Extrinsicler (Çağrılabilir Fonksiyonlar)

| Fonksiyon | Kim | Ne Yapar |
|-----------|-----|----------|
| `create_listing` | Satıcı | Yeni ilan (KOD veya TL fiyat + IBAN hash) |
| `cancel_listing` | Satıcı | İlanı iptal eder |
| `purchase` | Alıcı | Satın alır (KOD: fiyat+bond; TL: sadece bond + alıcı IBAN hash) |
| `accept_trade` | Satıcı | Kabul eder (TL ise AwaitingPayment, KOD ise Escrow) |
| `confirm_delivery` | Alıcı | KOD ticaretinde teslimatı onaylar |
| `mark_payment_sent` | Alıcı | TL havalesini yaptığını bildirir |
| `confirm_tl_payment` | Satıcı | TL ödemesinin geldiğini onaylar, ticaret tamamlanır |
| `open_dispute` | İkisi de | Anlaşmazlık başlatır |
| `resolve_dispute` | Admin | Anlaşmazlığı çözer |
| `set_kod_tl_rate` | Admin | KOD/TL kurunu ayarlar (kuruş, varsayılan 100 = 1:1) |
| `set_trading_paused` | Admin | Acil durum durdurma |
| `set_kod_only_block` | Admin | KOD-only bloğunu değiştirir |

### Varlık Sicili (Asset Registry)

Ticaret tamamlandığında ürün sahipliği otomatik olarak devredilir ve tam geçmiş on-chain'de saklanır.

| Storage | Açıklama |
|---------|----------|
| `assets(asset_id)` | Varlık bilgisi: current_owner, original_owner, transfer_count, asset_hash |
| `assetOwnershipHistory(asset_id)` | Sahiplik geçmişi: kimden kime, hangi ticaretle, ne zaman, fiyat |
| `ownerAssets(account)` | Kullanıcının sahip olduğu tüm varlık ID'leri |
| `assetByHash(hash)` | Ürün hash'inden asset_id lookup |

**Nasıl Çalışır:**
1. `create_listing` → Ürün otomatik olarak sicile kaydedilir (`AssetRegistered` event)
2. `confirm_delivery` / `confirm_tl_payment` → Sahiplik alıcıya geçer (`OwnershipTransferred` event)
3. `resolve_dispute` (alıcı kazanırsa) → Sahiplik alıcıya devredilir
4. İkinci el: Aynı ürün tekrar satışa çıkarılabilir, mevcut asset_id korunur

---

## ⛏️ Madencilik

### Başlamak İçin

```bash
# İndir ve çalıştır - bu kadar!
./kod-node --mine

# Otomatik cüzdan oluşturulur:
# 🔑 Address: 5Fxyz...
# 📁 Saved to: ~/.kod/wallet.json
```

### Cüzdan Yönetimi

```bash
# Cüzdan bilgisi
./kod-node wallet info

# Seed phrase göster (yedekle!)
./kod-node wallet export-seed

# Yeni cüzdan oluştur
./kod-node wallet new
```

### Teknik Detaylar

| Özellik | Değer |
|---------|-------|
| **Algoritma** | SHA3-256 |
| **Blok Süresi** | 6 saniye |
| **Zorluk Ayarı** | Dinamik (her blokta) |
| **Konsensus** | Proof of Work |
| **Miner = Full Node** | ✅ |

---

## 🛠️ Teknik Detaylar

### Teknoloji Yığını

```
┌─────────────────────────────────────────┐
│ Framework    │ Substrate (Polkadot SDK) │
│ Dil          │ Rust                     │
│ Runtime      │ WASM                     │
│ Veritabanı   │ RocksDB                  │
│ Ağ           │ libp2p                   │
│ Kriptografi  │ sr25519 (Schnorrkel)     │
│ Hashing      │ SHA3-256, Blake2         │
└─────────────────────────────────────────┘
```

### Pallet Yapısı

```
pallets/
├── block-reward/          # Madencilik ödülleri
│   ├── src/
│   │   ├── lib.rs         # Ana pallet kodu
│   │   └── inherent.rs    # Inherent data provider
│   └── Cargo.toml
│
└── trade/                 # Ticaret sistemi
    ├── src/
    │   └── lib.rs         # Ticaret pallet kodu
    └── Cargo.toml

runtime/
├── src/
│   ├── lib.rs             # Runtime tanımı
│   ├── configs/           # Pallet konfigürasyonları
│   └── apis.rs            # Runtime API'leri
└── Cargo.toml

node/
├── src/
│   ├── main.rs            # Giriş noktası
│   ├── service.rs         # Node servisleri + mining
│   ├── wallet.rs          # Cüzdan yönetimi
│   ├── cli.rs             # Komut satırı
│   └── chain_spec.rs      # Zincir spesifikasyonu
└── Cargo.toml
```

---

## 📈 Yol Haritası

### ✅ Tamamlandı (v7.0.0)

- [x] Proof of Work konsensus (SHA3-256)
- [x] Blok ödülleri (halving ile)
- [x] Otomatik cüzdan oluşturma
- [x] Trade pallet (gelişmiş)
  - [x] İlan oluşturma/iptal (conditions_root, device attestation hash, IPFS CID hash)
  - [x] Satın alma (escrow) → PendingSellerConfirm durumu
  - [x] **Satıcı kabul/red (accept_trade / reject_trade)**
  - [x] **On-chain cihaz tanılama (submit_diagnostics)**
  - [x] Teslimat onayı (diagnostik hash dahil)
  - [x] Anlaşmazlık açma/çözme (blockchain kanıtları ile)
  - [x] KOD-only modu
  - [x] **Şifreli sözleşme desteği (NaCl Box - x25519-xsalsa20-poly1305)**
  - [x] **EncryptedContracts storage (max 8KB)**
  - [x] **ContractEncryptionKeys storage (taraf başına şifreli anahtar)**
  - [x] **Sözleşme maddeleri (ClauseType + ContractClause)**
  - [x] **TL ödeme entegrasyonu** (AwaitingPayment, PaymentSent, IBAN hash)
  - [x] **Varlık Sicili (Asset Registry)** — sahiplik takibi on-chain
- [x] Multi-platform build (Linux, macOS, Windows)
- [x] Website (kod.services)
  - [x] Mining sayfası (download + quick start)
  - [x] **Explorer sayfası (trade sorgulama + sözleşme deşifreleme)**
- [x] **Mobil uygulama (React Native/Expo)**
  - [x] Blockchain entegrasyonu (@polkadot/api)
  - [x] PhoneDoctor tarzı 12 cihaz testi
  - [x] Otomatik QR oluşturma + tarama
  - [x] Trade detay: sözleşme hash, diagnostic rapor, tam blockchain verisi
  - [x] **NaCl Box şifreleme ile sözleşme gizliliği**
  - [x] **12 kelime ile sözleşme deşifreleme**
  - [x] **TL satış/alış akışı (IBAN, blake2 hash)**

### 🔄 Devam Eden

- [x] Trade pallet testleri (2 telefon ile gerçek ticaret testi)
- [x] Polkadot.js Apps entegrasyonu
- [x] Mobil uygulama (React Native/Expo)
- [ ] Mainnet hazırlığı

### 📋 Planlanan

#### Kısa Vade (1-2 Hafta)
- [ ] Bildirim sistemi (satıcıya yeni talep bildirimi)
- [ ] IPFS entegrasyonu (resim/detay depolama)

#### Orta Vade (1-2 Ay)
- [ ] Puan/değerlendirme sistemi
- [ ] Hakem sistemi (çoklu hakem, oylama)
- [ ] Kargo kontratı (time-lock)

#### Uzun Vade (3-6 Ay)
- [ ] Topluluk şablonları
- [ ] Çoklu dil desteği
- [ ] Governance (DAO)

---

## 🔐 Güvenlik Modeli

### Teminat (Bond) Sistemi

```
Dolandırıcılığı Önleme:

SATIICI                          ALICI
  │                                │
  │ 100 KOD teminat                │ 100 KOD teminat
  │ + ürün değeri                  │ + ürün fiyatı
  └────────────────┬───────────────┘
                   │
                   ▼
              🔒 ESCROW
              
Dürüst davranırsan: Teminatın geri döner ✅
Dolandırırsan: Teminatını kaybedersin ❌

→ Dolandırmanın maliyeti > kazancı = caydırıcı!
```

### Merkle Proof (Planlanan)

```
Sorun: 20 koşulu blockchain'e yazmak pahalı

Çözüm: Sadece ROOT HASH on-chain'de

┌─────────────────────────────────────────┐
│ On-chain: 32 byte (Merkle root)         │
│ Off-chain: Tüm koşullar (IPFS/DB)       │
│                                         │
│ Anlaşmazlıkta: Merkle proof ile doğrula │
└─────────────────────────────────────────┘
```

---

## 🚀 Hızlı Başlangıç

### Madenci Olarak

```bash
# 1. İndir
wget https://github.com/emirkeceli123/polkadot-sdk-solochain-template/releases/latest/download/kod-node-linux-x64.tar.gz

# 2. Çıkar
tar -xzf kod-node-linux-x64.tar.gz

# 3. Çalıştır
./kod-node --mine

# 4. Cüzdanını kontrol et
./kod-node wallet info
```

### Geliştirici Olarak

```bash
# 1. Klonla
git clone https://github.com/emirkeceli123/polkadot-sdk-solochain-template.git
cd polkadot-sdk-solochain-template

# 2. Derle
cargo build --release

# 3. Dev modda çalıştır
./target/release/kod-node --dev --mine --tmp

# 4. Polkadot.js Apps'e bağlan
# https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9944
```

---

## 📊 Karşılaştırma

### KOD vs Diğer Platformlar

| Özellik | KOD Chain | Ethereum | OpenBazaar | eBay |
|---------|-----------|----------|------------|------|
| Merkeziyetsiz | ✅ | ✅ | ✅ | ❌ |
| Proof of Work | ✅ | ❌ (PoS) | ❌ | - |
| Escrow | On-chain | Smart Contract | Multisig | Platform |
| Aracı | Yok | Gas | Yok | Platform |
| Sansür | Dayanıklı | Dayanıklı | Dayanıklı | Var |
| Node Gerekli | İsteğe bağlı | Genelde Infura | Evet | Hayır |
| Dispute | On-chain | Off-chain | 2-of-3 | Platform |

---

## 📜 Lisans

MIT License - Özgürce kullanın, değiştirin, dağıtın.

---

## 🤝 Katkıda Bulunma

1. Fork edin
2. Branch oluşturun (`git checkout -b feature/amazing`)
3. Commit edin (`git commit -m 'Add amazing feature'`)
4. Push edin (`git push origin feature/amazing`)
5. Pull Request açın

---

## 📞 İletişim

- **Website:** https://kod.services
- **GitHub:** https://github.com/emirkeceli123/polkadot-sdk-solochain-template
- **Explorer:** https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9944

---

<div align="center">

**KOD Chain** - *Güvenilir Ticaretin Geleceği* 🔗

</div>

