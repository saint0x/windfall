use std::sync::Arc;
use tokio::time::{sleep, Duration};
use crate::{
    AppState,
    db::operations,
    error::Result,
};
use aptos_sdk::types::account_address::AccountAddress;
use log::{info, error};

pub struct EventListener {
    state: Arc<AppState>,
    last_processed_sequence: u64,
}

impl EventListener {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            last_processed_sequence: 0,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting event listener service...");
        
        loop {
            if let Err(e) = self.process_events().await {
                error!("Error processing events: {}", e);
                sleep(Duration::from_secs(5)).await;
            }
            sleep(Duration::from_secs(1)).await;
        }
    }

    async fn process_events(&mut self) -> Result<()> {
        // Process Asset events
        self.process_asset_events().await?;
        // Process Governance events
        self.process_governance_events().await?;
        // Process Registry events
        self.process_registry_events().await?;

        Ok(())
    }

    async fn process_asset_events(&mut self) -> Result<()> {
        let events = self.state.client.get_account_events(
            AccountAddress::from_hex_literal("@windfall")?,
            "windfall::asset::AssetEvents",
            "transfer_events",
            Some(self.last_processed_sequence),
            Some(100),
        ).await?;

        for event in events {
            if let Ok(transfer_event) = serde_json::from_value::<AssetTransferEvent>(event) {
                // Update balances in database
                operations::update_balances(
                    &self.state.db,
                    &transfer_event.symbol,
                    &transfer_event.from,
                    &transfer_event.to,
                    transfer_event.amount,
                ).await?;
            }
        }

        Ok(())
    }

    async fn process_governance_events(&mut self) -> Result<()> {
        let events = self.state.client.get_account_events(
            AccountAddress::from_hex_literal("@windfall")?,
            "windfall::governance::GovernanceEvents",
            "proposal_events",
            Some(self.last_processed_sequence),
            Some(100),
        ).await?;

        for event in events {
            if let Ok(proposal_event) = serde_json::from_value::<ProposalEvent>(event) {
                match proposal_event.event_type.as_str() {
                    "created" => {
                        operations::sync_proposal_creation(
                            &self.state.db,
                            proposal_event.proposal_id,
                            &proposal_event.proposer,
                        ).await?;
                    },
                    "executed" => {
                        operations::sync_proposal_execution(
                            &self.state.db,
                            proposal_event.proposal_id,
                        ).await?;
                    },
                    "vetoed" => {
                        operations::sync_proposal_veto(
                            &self.state.db,
                            proposal_event.proposal_id,
                        ).await?;
                    },
                    _ => {}
                }
            }
        }

        Ok(())
    }

    async fn process_registry_events(&mut self) -> Result<()> {
        let events = self.state.client.get_account_events(
            AccountAddress::from_hex_literal("@windfall")?,
            "windfall::registry::RegistryEvents",
            "member_events",
            Some(self.last_processed_sequence),
            Some(100),
        ).await?;

        for event in events {
            if let Ok(member_event) = serde_json::from_value::<MemberEvent>(event) {
                match member_event.event_type.as_str() {
                    "added" => {
                        operations::sync_member_addition(
                            &self.state.db,
                            member_event.fund_id,
                            &member_event.member_address,
                        ).await?;
                    },
                    "removed" => {
                        operations::sync_member_removal(
                            &self.state.db,
                            member_event.fund_id,
                            &member_event.member_address,
                        ).await?;
                    },
                    _ => {}
                }
            }
        }

        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct AssetTransferEvent {
    symbol: String,
    from: String,
    to: String,
    amount: u64,
}

#[derive(serde::Deserialize)]
struct ProposalEvent {
    proposal_id: u64,
    proposer: String,
    event_type: String,
}

#[derive(serde::Deserialize)]
struct MemberEvent {
    fund_id: i64,
    member_address: String,
    event_type: String,
} 