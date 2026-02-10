# 🔗 KOD Chain

**Güvenilir, Merkeziyetsiz Ticaret Platformu**

[![Build](https://github.com/emirkeceli123/polkadot-sdk-solochain-template/actions/workflows/build-release.yml/badge.svg)](https://github.com/emirkeceli123/polkadot-sdk-solochain-template/actions)
[![Release](https://img.shields.io/github/v/release/emirkeceli123/polkadot-sdk-solochain-template)](https://github.com/emirkeceli123/polkadot-sdk-solochain-template/releases)

---

## 🎯 Nedir?

KOD Chain, **aracısız ve güvenilir ticaret** için tasarlanmış bir Proof-of-Work blockchain'dir.

- ⛏️ **Gerçek PoW Madenciliği** - SHA3-256 algoritması
- 🔒 **Escrow Sistemi** - Güvenli ticaret
- 💰 **1 Milyar Toplam Arz** - Sabit, enflasyonsuz
- 🌍 **Tamamen Merkeziyetsiz** - Herkes node çalıştırabilir

---

## ⚡ Hızlı Başlangıç

### Madencilik Başlat

```bash
# İndir
wget https://kod.services/download/kod-node-linux-x64.tar.gz

# Çıkar ve çalıştır
tar -xzf kod-node-*.tar.gz
chmod +x kod-node
./kod-node --mine

# Cüzdan otomatik oluşturulur! 🎉
```

### Cüzdan Yönetimi

```bash
# Cüzdan bilgisi
./kod-node wallet info

# Seed phrase (yedekle!)
./kod-node wallet export-seed
```

---

## 📊 Ekonomi

| Parametre | Değer |
|-----------|-------|
| **Toplam Arz** | 1,000,000,000 KOD |
| **Blok Ödülü** | 50 KOD |
| **Blok Süresi** | 6 saniye |
| **Halving** | Her 10.5M blok (~2 yıl) |
| **Algoritma** | SHA3-256 |

### Dağılım

```
Madencilik Havuzu: 900,000,000 KOD (%90)
Geliştirme:        100,000,000 KOD (%10)
```

---

## 🏪 Trade Pallet

Güvenli ticaret için yerleşik escrow sistemi:

```
Satıcı → İlan Ver → Alıcı Satın Al → Satıcı Kabul → Escrow / TL Havale Bekleme → Teslimat veya TL Onay → Ödeme
                                                       ↓
                                                  Anlaşmazlık? → Hakem Kararı (BC kanıtları ile)
```

### Özellikler

- ✅ İlan oluşturma/iptal
- ✅ Escrow ile satın alma
- ✅ **TL ödeme desteği** – Fiyat TL, ödeme havale; KOD sadece teminat (%10)
- ✅ **Satıcı kabul/red sistemi (PendingSellerConfirm)**
- ✅ Teslimat onayı (KOD) veya TL ödeme onayı (confirm_tl_payment)
- ✅ **Cihaz tanılama raporu (submit_diagnostics)** - on-chain test sonuçları
- ✅ Anlaşmazlık çözümü (blockchain kanıtları ile)
- ✅ Teminat sistemi
- ✅ KOD-only modu (blok 21M sonrası ~4 yıl)
- ✅ **Varlık Sicili (Asset Registry)** — ticaret tamamlanınca sahiplik otomatik devredilir, tam geçmiş on-chain

---

## 🛠️ Geliştirici Rehberi

### Kaynak Koddan Derleme

```bash
# Gereksinimler: Rust 1.76+, protoc, cmake

git clone https://github.com/emirkeceli123/polkadot-sdk-solochain-template.git
cd polkadot-sdk-solochain-template

cargo build --release
./target/release/kod-node --dev --mine --tmp
```

### Polkadot.js Apps ile Bağlanma

```
https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9944
```

---

## 📁 Proje Yapısı

```
kod-chain/
├── node/                 # Node (CLI, mining, network)
├── runtime/              # Runtime (WASM)
├── pallets/
│   ├── block-reward/     # Madencilik ödülleri
│   └── trade/            # Ticaret sistemi
├── website/              # kod.services
└── docs/                 # Dokümantasyon
```

---

## 🔗 Bağlantılar

- 🌐 **Website:** [kod.services](https://kod.services)
- 📖 **Dokümantasyon:** [docs/KOD_CHAIN.md](docs/KOD_CHAIN.md)
- 📦 **Releases:** [GitHub Releases](https://github.com/emirkeceli123/polkadot-sdk-solochain-template/releases)
- 📱 **Mobile App:** [kod-mobile/](../kod-mobile/) - React Native/Expo uygulaması

---

## 📱 Mobil Uygulama

KOD Chain için tam özellikli mobil ticaret uygulaması:

```bash
cd kod-mobile
npm install --legacy-peer-deps
npx expo start
```

**Özellikler:**
- ✅ Blockchain entegrasyonu (@polkadot/api)
- ✅ Otomatik cihaz tespiti
- ✅ Güvenli cüzdan yönetimi (sr25519)
- ✅ Gerçek zamanlı ilan görüntüleme
- ✅ Device attestation ve Merkle tree

Detaylar için: [kod-mobile/README.md](../kod-mobile/README.md)

---

## 📝 Changelog

### v7.0.0 - Varlık Sicili (Asset Registry) (2026-02-10)

**Blockchain:**
- ✅ **Asset Registry** — her ilan bir varlık (ürün) kaydı oluşturur
- ✅ Ticaret tamamlanınca sahiplik otomatik devredilir (satıcı → alıcı)
- ✅ Tam sahiplik geçmişi on-chain (`OwnershipRecord`)
- ✅ İkinci el desteği: aynı ürün tekrar satışa çıkarılabilir (mevcut asset_id kullanılır)
- ✅ Anlaşmazlık sonucu sahiplik devri (`DisputeResolution`)
- ✅ Yeni storage: `Assets`, `AssetOwnershipHistory`, `OwnerAssets`, `AssetByHash`
- ✅ Yeni eventler: `AssetRegistered`, `OwnershipTransferred`

### v6.1.0 - Bugfix: TL Trade Dispute & Bond (2026-02-10)

**Blockchain:**
- ✅ **resolve_dispute KRİTİK FIX**: TL trade'lerde buyer_reserved doğru hesaplanıyor
- ✅ **calculate_bond_from_tl**: integer division kaybı düzeltildi
- ✅ **confirm_delivery** açıklayıcı yorum

**Mobil:**
- ✅ IBAN hash blake2_256 ile (chain ile tutarlı)
- ✅ Alıcı IBAN UX iyileştirmesi (ayrı "Banka Bilgileri" bölümü)
- ✅ PaymentSent durumunda alıcıya bilgi kartı

### v6.0.0 - TL Ödeme + 6sn Blok + Mobil TL Akışı (2026-02)

**Blockchain:**
- ✅ **Blok süresi 6 saniye** (ekonomi aynı: 50 KOD/blok, halving 10.5M, KOD-only 21M)
- ✅ **TL ödeme entegrasyonu:** Fiyat TL (kuruş), KOD teminat %10; IBAN hash on-chain
- ✅ Yeni extrinsic'ler: `set_kod_tl_rate`, `mark_payment_sent`, `confirm_tl_payment`
- ✅ Yeni trade durumları: `AwaitingPayment`, `PaymentSent`
- ✅ Sözleşme maddeleri (ClauseType + ContractClause) – önceden eklendi

**Mobil (kod-mobile):**
- ✅ Satış ekranında TL/KOD seçimi, TL fiyat + IBAN girişi
- ✅ İlan detayda TL fiyat gösterimi, alıcı IBAN girişi
- ✅ Ticaret detayda TL akışı: "Ödeme gönderildi", "TL ödeme onayla"

### 2026-02-07 - Trade Pallet v2.0 + Mobile v1.1

**Yeni Ticaret Akışı:**
- ✅ **PendingSellerConfirm** durumu - satıcı kabul/red mekanizması
- ✅ **accept_trade** extrinsic - satıcı kabul eder, taraflar + şartlar BC'ye yazılır
- ✅ **reject_trade** extrinsic - satıcı reddeder, alıcıya iade yapılır
- ✅ **submit_diagnostics** extrinsic - cihaz test sonuçları on-chain kaydedilir
- ✅ **DiagnosticReport** + **DiagnosticTests** - her test ayrı ayrı BC'de
- ✅ **TradeAccepted/TradeRejected** eventleri
- ✅ **confirm_delivery** artık diagnostik hash'i de dahil eder

**Mobil Uygulama:**
- ✅ Satıcı kabul/red UI (trade detay ekranı)
- ✅ PhoneDoctor tarzı 12 cihaz testi (ekran, dokunmatik, hoparlör, mikrofon, titreşim, ivmeölçer, jiroskop, kamera, cihaz bilgisi)
- ✅ Otomatik QR oluşturma (testler bitince)
- ✅ QR tarama ve doğrulama (alıcı tarafı)
- ✅ Blockchain sözleşme detayları (contract_hash, final_hash, diagnostic rapor)
- ✅ Trade detay ekranında tam blockchain verisi görüntüleme
- ✅ useFocusEffect ile trade listesi otomatik yenileme

**Test Edildi:**
- 2 telefon ile gerçek ticaret testi (Xiaomi + iPhone)
- Node: `./kod-node --dev --mine --tmp --rpc-external --rpc-cors all`
- Test hesaplar: `//Alice` (satıcı), `//Bob` (alıcı)
- Tam akış: İlan → Satın Al → Satıcı Kabul → Test → QR → Onay → Tamamlandı

Detaylar: [kod-mobile/docs/TECHNICAL.md](../kod-mobile/docs/TECHNICAL.md)

---

## 📄 Lisans

MIT License

---

<div align="center">

**KOD Chain** - *Güvenilir Ticaretin Geleceği* 🔗

Built with ❤️ using [Polkadot SDK](https://github.com/paritytech/polkadot-sdk)

</div>
