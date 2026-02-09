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
| **Blok Ödülü** | 250 KOD |
| **Blok Süresi** | 30 saniye |
| **Halving** | Her 2.1M blok (~2 yıl) |
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
Satıcı → İlan Ver → Alıcı Satın Al → Satıcı Kabul → Escrow → Test → QR → Teslimat → Ödeme
                                                       ↓
                                                  Anlaşmazlık? → Hakem Kararı (BC kanıtları ile)
```

### Özellikler

- ✅ İlan oluşturma/iptal
- ✅ Escrow ile satın alma
- ✅ **Satıcı kabul/red sistemi (PendingSellerConfirm)**
- ✅ Teslimat onayı
- ✅ **Cihaz tanılama raporu (submit_diagnostics)** - on-chain test sonuçları
- ✅ Anlaşmazlık çözümü (blockchain kanıtları ile)
- ✅ Teminat sistemi
- ✅ KOD-only modu (4 yıl sonra)

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
