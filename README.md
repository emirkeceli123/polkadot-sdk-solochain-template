# KOD Chain

**A Proof-of-Work blockchain built on Substrate**

## 🎯 Overview

KOD Chain is a true PoW (Proof-of-Work) blockchain where anyone can become a miner by simply running the node software. No staking required, no validators to trust - just pure computational mining.

### Key Features

- **SHA3-256 Mining**: Real proof-of-work algorithm
- **Fixed Supply**: 4,000,000 KOD total supply
- **Block Rewards**: 1,000 KOD per block to miners
- **Fair Launch**: No pre-mine, no team allocation
- **Substrate Based**: Modern blockchain framework

## 🚀 Quick Start

### Download

Visit [kod.services](https://kod.services) to download the pre-built binaries for your platform:

- Linux x86_64
- macOS Apple Silicon (M1/M2/M3/M4)
- macOS Intel
- Windows x86_64

### Start Mining

```bash
# Linux/macOS
tar -xzf kod-node-*.tar.gz
chmod +x kod-node
./kod-node --mine --reward-address YOUR_SS58_ADDRESS

# Windows
kod-node.exe --mine --reward-address YOUR_SS58_ADDRESS
```

### CLI Options

| Flag | Description |
|------|-------------|
| `--mine` | Enable mining mode |
| `--mining-threads N` | Number of mining threads (default: 1) |
| `--reward-address ADDR` | SS58 address for mining rewards |
| `--name NAME` | Human-readable node name |
| `--base-path PATH` | Data directory |
| `--port PORT` | P2P port (default: 30333) |
| `--rpc-port PORT` | RPC port (default: 9944) |

### Example Commands

```bash
# Single-threaded mining
./kod-node --mine --reward-address 5GrwvaEF...

# Multi-threaded mining (4 threads)
./kod-node --mine --mining-threads 4 --reward-address 5GrwvaEF...

# Run as a bootnode
./kod-node --name "KOD-Bootnode" --port 30333

# Connect to existing network
./kod-node --mine --bootnodes /ip4/IP/tcp/30333/p2p/PEER_ID --reward-address 5GrwvaEF...
```

## 🏗️ Building from Source

### Prerequisites

- Rust 1.75+ (install from [rustup.rs](https://rustup.rs))
- CMake
- Protobuf compiler (`protoc`)
- LLVM/Clang

### Build

```bash
# Clone the repository
git clone https://github.com/emirkeceli123/polkadot-sdk-solochain-template.git
cd polkadot-sdk-solochain-template

# Build in release mode
cargo build --release

# Binary will be at: target/release/kod-node
```

## 📊 Economics

| Parameter | Value |
|-----------|-------|
| Total Supply | 4,000,000 KOD |
| Block Reward | 1,000 KOD |
| Target Block Time | ~60 seconds |
| Mining Algorithm | SHA3-256 |
| Decimal Places | 18 |

### Distribution

- **Mining Reserve**: 3,000,000 KOD (for block rewards)
- **Initial Distribution**: 1,000,000 KOD (development/community)

Mining will continue until the Mining Reserve is depleted (approximately 3,000 blocks).

## 🔐 Creating a Wallet

1. Visit [polkadot.js.org/apps](https://polkadot.js.org/apps)
2. Go to Accounts → Add Account
3. Save your seed phrase securely!
4. Copy your SS58 address (starts with `5...`)
5. Use this address with `--reward-address`

## 📁 Project Structure

```
kod-chain/
├── node/               # Node implementation (CLI, networking, consensus)
├── runtime/            # Blockchain runtime (FRAME pallets)
├── pallets/
│   ├── template/       # Example pallet
│   └── block-reward/   # Mining reward distribution
├── website/            # Landing page for kod.services
└── .github/workflows/  # CI/CD for multi-platform builds
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

MIT-0 License - Use however you want!

---

Built with ❤️ using [Polkadot SDK](https://github.com/paritytech/polkadot-sdk)
