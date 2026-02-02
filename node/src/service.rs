//! Service implementation for KOD Chain.
//!
//! This module sets up the PoW consensus and mining infrastructure.

use futures::FutureExt;
use sc_client_api::{Backend, BlockImport};
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use kod_runtime::{self, apis::RuntimeApi, opaque::Block};
use std::sync::Arc;
use sp_runtime::traits::Block as BlockT;
use sp_consensus::BlockOrigin;

pub(crate) type FullClient = sc_service::TFullClient<
    Block,
    RuntimeApi,
    sc_executor::WasmExecutor<sp_io::SubstrateHostFunctions>,
>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

pub type Service = sc_service::PartialComponents<
    FullClient,
    FullBackend,
    FullSelectChain,
    sc_consensus::DefaultImportQueue<Block>,
    sc_transaction_pool::TransactionPoolHandle<Block, FullClient>,
    Option<Telemetry>,
>;

/// Create the partial components for the node.
pub fn new_partial(config: &Configuration) -> Result<Service, ServiceError> {
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = sc_service::new_wasm_executor::<sp_io::SubstrateHostFunctions>(&config.executor);
    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager.spawn_handle().spawn("telemetry", None, worker.run());
        telemetry
    });

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = Arc::from(
        sc_transaction_pool::Builder::new(
            task_manager.spawn_essential_handle(),
            client.clone(),
            config.role.is_authority().into(),
        )
        .with_options(config.transaction_pool.clone())
        .with_prometheus(config.prometheus_registry())
        .build(),
    );

    // For PoW, we use a simple import queue that doesn't verify PoW
    // (PoW verification happens during mining/import)
    let import_queue = sc_consensus::BasicQueue::new(
        PowVerifier {},
        Box::new(client.clone()),
        None,
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
    );

    Ok(sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: telemetry,
    })
}

/// Simple PoW block verifier (accepts all blocks for now)
struct PowVerifier;

#[async_trait::async_trait]
impl<B: sp_runtime::traits::Block> sc_consensus::Verifier<B> for PowVerifier {
    async fn verify(
        &self,
        block: sc_consensus::BlockImportParams<B>,
    ) -> Result<sc_consensus::BlockImportParams<B>, String> {
        let mut block = block;
        block.fork_choice = Some(sc_consensus::ForkChoiceStrategy::LongestChain);
        Ok(block)
    }
}

/// Builds a new service for a full client.
pub fn new_full<N: sc_network::NetworkBackend<Block, <Block as sp_runtime::traits::Block>::Hash>>(
    config: Configuration,
    mine: bool,
    mining_threads: usize,
    reward_address: Option<String>,
) -> Result<TaskManager, ServiceError> {
    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: mut telemetry,
    } = new_partial(&config)?;

    let net_config = sc_network::config::FullNetworkConfiguration::<
        Block,
        <Block as sp_runtime::traits::Block>::Hash,
        N,
    >::new(&config.network, config.prometheus_registry().cloned());
    let metrics = N::register_notification_metrics(config.prometheus_registry());

    let (network, system_rpc_tx, tx_handler_controller, sync_service) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            net_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync_config: None, // No warp sync for PoW
            block_relay: None,
            metrics,
        })?;

    if config.offchain_worker.enabled {
        let offchain_workers =
            sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
                runtime_api_provider: client.clone(),
                is_validator: config.role.is_authority(),
                keystore: Some(keystore_container.keystore()),
                offchain_db: backend.offchain_storage(),
                transaction_pool: Some(OffchainTransactionPoolFactory::new(
                    transaction_pool.clone(),
                )),
                network_provider: Arc::new(network.clone()),
                enable_http_requests: true,
                custom_extensions: |_| vec![],
            })?;
        task_manager.spawn_handle().spawn(
            "offchain-workers-runner",
            "offchain-worker",
            offchain_workers.run(client.clone(), task_manager.spawn_handle()).boxed(),
        );
    }

    let rpc_extensions_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();

        Box::new(move |_| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
            };
            crate::rpc::create_full(deps).map_err(Into::into)
        })
    };

    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network: Arc::new(network.clone()),
        client: client.clone(),
        keystore: keystore_container.keystore(),
        task_manager: &mut task_manager,
        transaction_pool: transaction_pool.clone(),
        rpc_builder: rpc_extensions_builder,
        backend: backend.clone(),
        system_rpc_tx,
        tx_handler_controller,
        sync_service: sync_service.clone(),
        config,
        telemetry: telemetry.as_mut(),
    })?;

    // Start mining if enabled
    if mine {
        log::info!("⛏️  Mining enabled with {} thread(s)", mining_threads);
        if let Some(ref addr) = reward_address {
            log::info!("💰 Mining rewards will be sent to: {}", addr);
        } else {
            log::warn!("⚠️  No reward address specified. Mining but rewards won't be claimed!");
        }

        // Create block proposer factory
        let proposer_factory = sc_basic_authorship::ProposerFactory::new(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool.clone(),
            None, // No prometheus for now
            None, // No telemetry
        );

        // Spawn mining task
        let mining_client = client.clone();
        let mining_select_chain = select_chain.clone();
        let mining_reward_address = reward_address.clone();
        
        task_manager.spawn_essential_handle().spawn_blocking(
            "pow-miner",
            Some("mining"),
            Box::pin(async move {
                mining_loop(
                    mining_client,
                    proposer_factory,
                    mining_select_chain,
                    mining_reward_address,
                ).await;
            }),
        );
    }

    Ok(task_manager)
}

/// Main mining loop - produces blocks with PoW
async fn mining_loop<PF, SC>(
    client: Arc<FullClient>,
    mut proposer_factory: PF,
    select_chain: SC,
    reward_address: Option<String>,
) where
    PF: sp_consensus::Environment<Block> + Send + 'static,
    PF::Proposer: sp_consensus::Proposer<Block>,
    SC: sp_consensus::SelectChain<Block> + 'static,
{
    use sha3::{Digest, Sha3_256};
    use sp_runtime::traits::Header;
    use sc_client_api::BlockBackend;
    
    log::info!("🔨 Mining loop started");
    
    let difficulty: u128 = 100_000; // Lower difficulty for dev testing
    let mut nonce: u64 = rand::random();
    let mut blocks_mined: u64 = 0;
    let mut last_log = std::time::Instant::now();
    let mut hash_count: u64 = 0;
    
    // Block time target: 6 seconds
    let block_time = std::time::Duration::from_secs(6);
    let mut last_block_time = std::time::Instant::now();
    
    loop {
        // Get the best block
        let best_header = match select_chain.best_chain().await {
            Ok(header) => header,
            Err(e) => {
                log::error!("Failed to get best chain: {:?}", e);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }
        };
        
        let best_hash = best_header.hash();
        let best_number = *best_header.number();
        
        // Create mining data
        let mining_data = format!(
            "{}:{}:{}:{}",
            hex::encode(best_hash.as_ref()),
            best_number + 1,
            reward_address.as_deref().unwrap_or(""),
            nonce
        );
        
        // Hash the data
        let mut hasher = Sha3_256::new();
        hasher.update(mining_data.as_bytes());
        let hash = hasher.finalize();
        
        hash_count += 1;
        nonce = nonce.wrapping_add(1);
        
        // Check if hash meets difficulty
        let hash_value = u128::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0,
            hash[0], hash[1], hash[2], hash[3],
            hash[4], hash[5], hash[6], hash[7],
        ]);
        
        // Check if we should produce a block (difficulty met + enough time passed)
        let time_ok = last_block_time.elapsed() >= block_time;
        let pow_ok = hash_value < u128::MAX / difficulty;
        
        if pow_ok && time_ok {
            log::info!(
                "🎉 Found valid PoW! Creating block #{}, nonce={}, hash={:x}",
                best_number + 1,
                nonce,
                hash_value
            );
            
            // Create the block using proposer factory
            match create_block_with_proposer(
                &client,
                &mut proposer_factory,
                &best_header,
            ).await {
                Ok(()) => {
                    blocks_mined += 1;
                    last_block_time = std::time::Instant::now();
                    log::info!(
                        "✅ Block #{} successfully produced and imported!",
                        best_number + 1
                    );
                }
                Err(e) => {
                    log::error!("❌ Failed to create block: {}", e);
                    // Wait a bit before retrying
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
            
            // Reset nonce after block
            nonce = rand::random();
        }
        
        // Log stats every 10 seconds
        if last_log.elapsed() > std::time::Duration::from_secs(10) {
            let hash_rate = hash_count as f64 / last_log.elapsed().as_secs_f64();
            log::info!(
                "⛏️  Mining: {:.2} H/s, {} blocks produced, current best #{}",
                hash_rate,
                blocks_mined,
                best_number
            );
            hash_count = 0;
            last_log = std::time::Instant::now();
        }
        
        // Yield to other tasks occasionally
        if nonce % 10000 == 0 {
            tokio::task::yield_now().await;
        }
    }
}

/// Create a block using the proposer factory and import it
async fn create_block_with_proposer<PF>(
    client: &Arc<FullClient>,
    proposer_factory: &mut PF,
    parent: &<Block as BlockT>::Header,
) -> Result<(), String>
where
    PF: sp_consensus::Environment<Block>,
    PF::Proposer: sp_consensus::Proposer<Block>,
{
    use sp_consensus::{Environment, Proposer};
    use sp_runtime::traits::Header;
    use sp_runtime::generic::Digest;
    
    let parent_hash = parent.hash();
    
    // Create inherent data (timestamp)
    let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
    let mut inherent_data = sp_inherents::InherentData::new();
    inherent_data.put_data(sp_timestamp::INHERENT_IDENTIFIER, &timestamp.timestamp())
        .map_err(|e| format!("Failed to add timestamp: {:?}", e))?;
    
    // Create proposer
    let proposer = proposer_factory
        .init(parent)
        .await
        .map_err(|e| format!("Failed to create proposer: {:?}", e))?;
    
    // Propose a block
    let proposal = proposer
        .propose(
            inherent_data,
            Digest::default(),
            std::time::Duration::from_secs(10),
            None,
        )
        .await
        .map_err(|e| format!("Failed to propose block: {:?}", e))?;
    
    let block = proposal.block;
    let header = block.header().clone();
    let body = block.extrinsics().to_vec();
    
    log::info!("📦 Proposed block with {} extrinsics", body.len());
    
    // Import the block
    let mut import_params = sc_consensus::BlockImportParams::new(BlockOrigin::Own, header);
    import_params.body = Some(body);
    import_params.fork_choice = Some(sc_consensus::ForkChoiceStrategy::LongestChain);
    import_params.state_action = sc_consensus::StateAction::ApplyChanges(
        sc_consensus::StorageChanges::Changes(proposal.storage_changes)
    );
    
    client.import_block(import_params)
        .await
        .map_err(|e| format!("Failed to import block: {:?}", e))?;
    
    Ok(())
}
