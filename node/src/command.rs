//! Command handling for KOD Chain node.

use crate::{
    chain_spec,
    cli::{Cli, Subcommand, WalletSubcommand},
    service,
    wallet,
};
use sc_cli::SubstrateCli;
use sc_service::PartialComponents;
use kod_runtime::Block;

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "KOD Node".into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "https://kod.services".into()
    }

    fn copyright_start_year() -> i32 {
        2025
    }

    fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(match id {
            "dev" => Box::new(chain_spec::development_chain_spec()?),
            "" | "local" => Box::new(chain_spec::local_chain_spec()?),
            "mainnet" => Box::new(chain_spec::mainnet_chain_spec()?),
            path => Box::new(chain_spec::ChainSpec::from_json_file(
                std::path::PathBuf::from(path),
            )?),
        })
    }
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
    let cli = Cli::from_args();

    match &cli.subcommand {
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, config.chain_spec), task_manager))
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    backend,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, backend, None), task_manager))
            })
        }
        Some(Subcommand::ChainInfo(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run::<Block>(&config))
        }
        Some(Subcommand::Wallet(wallet_cmd)) => {
            match wallet_cmd {
                WalletSubcommand::Info => {
                    wallet::print_wallet_info()
                        .map_err(|e| sc_cli::Error::Application(e.into()))?;
                    Ok(())
                }
                WalletSubcommand::New => {
                    if wallet::wallet_exists() {
                        println!("⚠️  WARNING: A wallet already exists!");
                        println!("   Existing wallet will be OVERWRITTEN.");
                        println!("   Press Ctrl+C to cancel, or wait 5 seconds to continue...");
                        std::thread::sleep(std::time::Duration::from_secs(5));
                    }
                    match wallet::generate_wallet() {
                        Ok(info) => {
                            println!("✅ New wallet created!");
                            println!("📍 Address: {}", info.address);
                            println!("📁 Saved to: ~/.kod/wallet.json");
                            println!("");
                            println!("⚠️  IMPORTANT: Backup your wallet.json file!");
                        }
                        Err(e) => {
                            println!("❌ Failed to create wallet: {}", e);
                        }
                    }
                    Ok(())
                }
                WalletSubcommand::ExportSeed => {
                    wallet::export_seed()
                        .map_err(|e| sc_cli::Error::Application(e.into()))?;
                    Ok(())
                }
            }
        }
        None => {
            let runner = cli.create_runner(&cli.run)?;
            let mine = cli.mine;
            let mining_threads = cli.mining_threads;
            
            // Auto-generate wallet if mining is enabled but no address provided
            let reward_address = if mine && cli.reward_address.is_none() {
                match wallet::get_or_create_wallet() {
                    Ok(addr) => Some(addr),
                    Err(e) => {
                        log::error!("Failed to create wallet: {}", e);
                        log::warn!("Mining will continue but rewards will not be claimed.");
                        None
                    }
                }
            } else {
                cli.reward_address.clone()
            };
            
            runner.run_node_until_exit(|config| async move {
                match config.network.network_backend.unwrap_or_default() {
                    sc_network::config::NetworkBackendType::Libp2p => service::new_full::<
                        sc_network::NetworkWorker<
                            kod_runtime::opaque::Block,
                            <kod_runtime::opaque::Block as sp_runtime::traits::Block>::Hash,
                        >,
                    >(config, mine, mining_threads, reward_address)
                    .map_err(sc_cli::Error::Service),
                    sc_network::config::NetworkBackendType::Litep2p => {
                        service::new_full::<sc_network::Litep2pNetworkBackend>(
                            config,
                            mine,
                            mining_threads,
                            reward_address,
                        )
                        .map_err(sc_cli::Error::Service)
                    }
                }
            })
        }
    }
}
