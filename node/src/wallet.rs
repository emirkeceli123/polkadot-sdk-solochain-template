//! Automatic wallet generation for KOD miners.
//!
//! This module handles:
//! - Generating new sr25519 keypairs
//! - Saving wallet info to ~/.kod/wallet.json
//! - Loading existing wallets

use sp_core::{sr25519, Pair, crypto::Ss58Codec};
use std::fs;
use std::path::PathBuf;

/// Wallet information
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct WalletInfo {
    /// SS58 encoded address
    pub address: String,
    /// Mnemonic seed phrase (12 words)
    pub seed_phrase: String,
    /// Creation timestamp
    pub created_at: String,
    /// Network name
    pub network: String,
}

/// Get the wallet directory path (~/.kod/)
pub fn wallet_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".kod")
}

/// Get the wallet file path (~/.kod/wallet.json)
pub fn wallet_path() -> PathBuf {
    wallet_dir().join("wallet.json")
}

/// Check if a wallet already exists
pub fn wallet_exists() -> bool {
    wallet_path().exists()
}

/// Generate a new wallet and save it
pub fn generate_wallet() -> Result<WalletInfo, String> {
    // Generate 16 bytes of entropy (128 bits = 12 word mnemonic)
    let mut entropy = [0u8; 16];
    getrandom::getrandom(&mut entropy)
        .map_err(|e| format!("Failed to generate entropy: {:?}", e))?;
    
    // Create mnemonic from entropy
    let mnemonic = bip39::Mnemonic::from_entropy(&entropy)
        .map_err(|e| format!("Failed to create mnemonic: {:?}", e))?;
    
    let seed_phrase = mnemonic.to_string();
    
    // Derive keypair from mnemonic
    let (pair, _) = sr25519::Pair::from_phrase(&seed_phrase, None)
        .map_err(|e| format!("Failed to create keypair: {:?}", e))?;
    
    // Get SS58 address
    let address = pair.public().to_ss58check();
    
    // Create wallet info
    let wallet = WalletInfo {
        address,
        seed_phrase,
        created_at: chrono::Utc::now().to_rfc3339(),
        network: "KOD".to_string(),
    };
    
    // Ensure directory exists
    let dir = wallet_dir();
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create wallet directory: {:?}", e))?;
    
    // Save wallet
    let json = serde_json::to_string_pretty(&wallet)
        .map_err(|e| format!("Failed to serialize wallet: {:?}", e))?;
    
    let path = wallet_path();
    fs::write(&path, &json)
        .map_err(|e| format!("Failed to save wallet: {:?}", e))?;
    
    // Set file permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path)
            .map_err(|e| format!("Failed to get file metadata: {:?}", e))?
            .permissions();
        perms.set_mode(0o600); // Only owner can read/write
        fs::set_permissions(&path, perms)
            .map_err(|e| format!("Failed to set file permissions: {:?}", e))?;
    }
    
    Ok(wallet)
}

/// Load existing wallet
pub fn load_wallet() -> Result<WalletInfo, String> {
    let path = wallet_path();
    let json = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read wallet file: {:?}", e))?;
    
    let wallet: WalletInfo = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse wallet file: {:?}", e))?;
    
    Ok(wallet)
}

/// Get or create wallet - returns the address
pub fn get_or_create_wallet() -> Result<String, String> {
    if wallet_exists() {
        let wallet = load_wallet()?;
        log::info!("💼 Loaded existing wallet");
        log::info!("📍 Address: {}", wallet.address);
        log::info!("📁 Wallet file: {:?}", wallet_path());
        Ok(wallet.address)
    } else {
        log::info!("🔑 Creating new miner wallet...");
        let wallet = generate_wallet()?;
        log::info!("");
        log::info!("╔══════════════════════════════════════════════════════════════╗");
        log::info!("║  🎉 NEW WALLET CREATED!                                      ║");
        log::info!("╠══════════════════════════════════════════════════════════════╣");
        log::info!("║  📍 Address: {}  ║", &wallet.address[..20]);
        log::info!("║  📁 Saved to: ~/.kod/wallet.json                             ║");
        log::info!("╠══════════════════════════════════════════════════════════════╣");
        log::info!("║  ⚠️  IMPORTANT: Backup your wallet.json file!                ║");
        log::info!("║  ⚠️  If you lose it, you lose your coins!                    ║");
        log::info!("╚══════════════════════════════════════════════════════════════╝");
        log::info!("");
        Ok(wallet.address)
    }
}

/// Print wallet info to console
pub fn print_wallet_info() -> Result<(), String> {
    if !wallet_exists() {
        println!("❌ No wallet found. Run with --mine to create one automatically.");
        return Ok(());
    }
    
    let wallet = load_wallet()?;
    
    println!("");
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  💼 KOD WALLET INFO                                          ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  📍 Address:                                                 ║");
    println!("║     {}  ║", wallet.address);
    println!("║                                                              ║");
    println!("║  📅 Created: {}                        ║", &wallet.created_at[..10]);
    println!("║  🌐 Network: {}                                            ║", wallet.network);
    println!("║  📁 File: ~/.kod/wallet.json                                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!("");
    
    Ok(())
}

/// Export seed phrase (for backup)
pub fn export_seed() -> Result<(), String> {
    if !wallet_exists() {
        println!("❌ No wallet found.");
        return Ok(());
    }
    
    let wallet = load_wallet()?;
    
    println!("");
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  🔐 SEED PHRASE - KEEP THIS SECRET!                          ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║                                                              ║");
    println!("║  {}  ║", wallet.seed_phrase);
    println!("║                                                              ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  ⚠️  Anyone with this phrase can steal your coins!           ║");
    println!("║  ⚠️  Write it down and store it safely!                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!("");
    
    Ok(())
}

