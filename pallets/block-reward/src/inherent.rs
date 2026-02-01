//! Inherent data provider for block rewards.
//! This runs on the node side to provide miner address to the runtime.

use crate::{MinerInherentData, INHERENT_IDENTIFIER};
use sp_inherents::{InherentData, InherentDataProvider as InherentDataProviderTrait};

/// Inherent data provider that provides the miner's reward address to the runtime.
pub struct BlockRewardInherentDataProvider {
    /// The SS58 address of the miner (as bytes)
    miner_address: Option<Vec<u8>>,
    /// Current block number
    block_number: u32,
}

impl BlockRewardInherentDataProvider {
    /// Create a new inherent data provider.
    ///
    /// # Arguments
    /// * `miner_address` - Optional SS58 address string of the miner
    /// * `block_number` - The block number being authored
    pub fn new(miner_address: Option<String>, block_number: u32) -> Self {
        Self {
            miner_address: miner_address.map(|s| s.into_bytes()),
            block_number,
        }
    }
}

#[async_trait::async_trait]
impl InherentDataProviderTrait for BlockRewardInherentDataProvider {
    async fn provide_inherent_data(
        &self,
        inherent_data: &mut InherentData,
    ) -> Result<(), sp_inherents::Error> {
        let data = MinerInherentData {
            miner_address: self.miner_address.clone(),
            block_number: self.block_number,
        };
        
        inherent_data.put_data(INHERENT_IDENTIFIER, &data)
    }

    async fn try_handle_error(
        &self,
        _identifier: &sp_inherents::InherentIdentifier,
        _error: &[u8],
    ) -> Option<Result<(), sp_inherents::Error>> {
        None
    }
}
