use crate::{
    AppState,
    error::Result,
    db::operations,
};
use aptos_sdk::types::account_address::AccountAddress;
use log::{info, error, warn};
use tokio::time::{sleep, Duration};
use std::str::FromStr;

pub struct BlockchainSynchronizer {
    state: AppState,
    sync_interval: Duration,
}

impl BlockchainSynchronizer {
    pub fn new(state: AppState, sync_interval: Duration) -> Self {
        Self {
            state,
            sync_interval,
        }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting blockchain synchronizer");
        loop {
            match self.sync_state().await {
                Ok(_) => {
                    info!("State synchronization completed successfully");
                }
                Err(e) => {
                    error!("Error during state synchronization: {}", e);
                }
            }
            sleep(self.sync_interval).await;
        }
    }

    async fn sync_state(&self) -> Result<()> {
        // Sync fund states
        self.sync_fund_states().await?;
        // Sync member states
        self.sync_member_states().await?;
        // Sync asset states
        self.sync_asset_states().await?;
        Ok(())
    }

    async fn sync_fund_states(&self) -> Result<()> {
        info!("Syncing fund states with blockchain");
        // Get all funds from database
        let funds = operations::get_all_funds(&self.state.db).await?;
        
        for fund in funds {
            // Get fund state from blockchain
            let fund_address = AccountAddress::from_str(&fund.executor_address)?;
            let fund_resource = self.state.client
                .get_resource::<FundResource>(fund_address, "0x1::windfall::fund::FundResource")
                .await;

            match fund_resource {
                Ok(resource) => {
                    // Update local state if needed
                    if resource.version > fund.version as u64 {
                        operations::update_fund_state(
                            &self.state.db,
                            fund.id,
                            resource.version,
                            resource.status,
                        ).await?;
                    }
                }
                Err(e) => {
                    // Only log as warning for resource not found, as this is expected for new funds
                    if e.to_string().contains("ResourceNotFound") {
                        warn!("Fund resource not found for {}: {}", fund.id, e);
                    } else {
                        error!("Failed to get fund resource for {}: {}", fund.id, e);
                    }
                    continue;
                }
            }
        }
        Ok(())
    }

    async fn sync_member_states(&self) -> Result<()> {
        info!("Syncing member states with blockchain");
        // Get all funds from database
        let funds = operations::get_all_funds(&self.state.db).await?;
        
        for fund in funds {
            // Get member list from blockchain
            let fund_address = AccountAddress::from_str(&fund.executor_address)?;
            let members_resource = self.state.client
                .get_resource::<MembersResource>(fund_address, "0x1::windfall::fund::MembersResource")
                .await;

            match members_resource {
                Ok(resource) => {
                    // Update local member states
                    for member in resource.members {
                        operations::sync_member_state(
                            &self.state.db,
                            fund.id,
                            &member.address,
                            member.share,
                            member.status,
                        ).await?;
                    }
                }
                Err(e) => {
                    // Only log as warning for resource not found
                    if e.to_string().contains("ResourceNotFound") {
                        warn!("Members resource not found for fund {}: {}", fund.id, e);
                    } else {
                        error!("Failed to get members resource for fund {}: {}", fund.id, e);
                    }
                    continue;
                }
            }
        }
        Ok(())
    }

    async fn sync_asset_states(&self) -> Result<()> {
        info!("Syncing asset states with blockchain");
        // Get all assets from database
        let assets = operations::get_all_assets(&self.state.db).await?;
        
        for asset in assets {
            // Skip assets without addresses
            let Some(address) = &asset.address else {
                continue;
            };
            
            // Get asset state from blockchain
            let asset_address = AccountAddress::from_str(address)?;
            let asset_resource = self.state.client
                .get_resource::<AssetResource>(asset_address, "0x1::windfall::asset::AssetResource")
                .await;

            match asset_resource {
                Ok(resource) => {
                    // Update local state if needed
                    if resource.version > asset.version as u64 {
                        operations::update_asset_state(
                            &self.state.db,
                            asset.id,
                            resource.version,
                            resource.total_supply,
                            resource.holders,
                        ).await?;
                    }
                }
                Err(e) => {
                    // Only log as warning for resource not found
                    if e.to_string().contains("ResourceNotFound") {
                        warn!("Asset resource not found for {}: {}", asset.id, e);
                    } else {
                        error!("Failed to get asset resource for {}: {}", asset.id, e);
                    }
                    continue;
                }
            }
        }
        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct FundResource {
    version: u64,
    status: String,
}

#[derive(serde::Deserialize)]
struct MembersResource {
    members: Vec<MemberInfo>,
}

#[derive(serde::Deserialize)]
struct MemberInfo {
    address: String,
    share: u64,
    status: String,
}

#[derive(serde::Deserialize)]
struct AssetResource {
    version: u64,
    total_supply: u64,
    holders: Vec<HolderInfo>,
}

#[derive(serde::Deserialize)]
pub struct HolderInfo {
    pub address: String,
    pub balance: u64,
} 