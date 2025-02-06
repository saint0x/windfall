use sqlx::sqlite::SqlitePool;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use anyhow::Context;
use crate::db::types::DbDateTime;
use crate::error::Result;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Fund {
    pub id: i64,
    pub name: String,
    pub executor_address: String,
    pub version: i64,
    pub status: String,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FundMember {
    pub id: i64,
    pub fund_id: i64,
    pub member_address: String,
    pub share: i64,
    pub status: String,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FundWallet {
    pub id: i64,
    pub fund_id: i64,
    pub wallet_address: String,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Investment {
    pub id: i64,
    pub fund_id: i64,
    pub asset_id: i64,
    pub amount: i64,
    pub withdrawn_amount: i64,
    pub investor_address: String,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: i64,
    pub fund_id: i64,
    pub sender_address: String,
    pub content: String,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Position {
    pub id: i64,
    pub fund_id: i64,
    pub asset_id: i64,
    pub size: i64,
    pub entry_price: i64,
    pub is_long: bool,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Proposal {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub end_time: DbDateTime,
    pub executed: bool,
    pub vetoed: bool,
    pub chain_id: i64,
    pub synced: bool,
    pub proposer_address: Option<String>,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Vote {
    pub id: i64,
    pub proposal_id: i64,
    pub voter_address: String,
    pub vote_type: bool,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Asset {
    pub id: i64,
    pub symbol: String,
    pub name: String,
    pub decimals: i32,
    pub version: i64,
    pub address: Option<String>,
    pub total_supply: i64,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Balance {
    pub id: i64,
    pub asset_id: i64,
    pub holder_address: String,
    pub amount: i64,
    pub created_at: DbDateTime,
    pub updated_at: DbDateTime,
}

pub async fn initialize_database(pool: &SqlitePool) -> Result<()> {
    // Enable foreign keys
    sqlx::query!("PRAGMA foreign_keys = ON;")
        .execute(pool)
        .await
        .context("Failed to enable foreign keys")?;

    // Create tables in the correct order (parents first)
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS funds (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            executor_address TEXT NOT NULL,
            version INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'active',
            description TEXT NOT NULL DEFAULT '',
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS assets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            symbol TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            decimals INTEGER NOT NULL,
            version INTEGER NOT NULL DEFAULT 0,
            address TEXT,
            total_supply INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS proposals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            end_time DATETIME NOT NULL,
            executed BOOLEAN NOT NULL DEFAULT FALSE,
            vetoed BOOLEAN NOT NULL DEFAULT FALSE,
            chain_id INTEGER NOT NULL DEFAULT 0,
            synced BOOLEAN NOT NULL DEFAULT FALSE,
            proposer_address TEXT,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#
    )
    .execute(pool)
    .await
    .context("Failed to create primary tables")?;

    // Create tables with foreign key constraints
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS fund_members (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fund_id INTEGER NOT NULL,
            member_address TEXT NOT NULL,
            share INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'active',
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (fund_id) REFERENCES funds(id),
            UNIQUE(fund_id, member_address)
        );

        CREATE TABLE IF NOT EXISTS fund_wallets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fund_id INTEGER NOT NULL,
            wallet_address TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (fund_id) REFERENCES funds(id),
            UNIQUE(fund_id)
        );

        CREATE TABLE IF NOT EXISTS investments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fund_id INTEGER NOT NULL,
            asset_id INTEGER NOT NULL,
            amount INTEGER NOT NULL,
            withdrawn_amount INTEGER NOT NULL DEFAULT 0,
            investor_address TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (fund_id) REFERENCES funds(id),
            FOREIGN KEY (asset_id) REFERENCES assets(id)
        );

        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fund_id INTEGER NOT NULL,
            sender_address TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (fund_id) REFERENCES funds(id)
        );

        CREATE TABLE IF NOT EXISTS positions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fund_id INTEGER NOT NULL,
            asset_id INTEGER NOT NULL,
            size INTEGER NOT NULL,
            entry_price INTEGER NOT NULL,
            is_long BOOLEAN NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (fund_id) REFERENCES funds(id),
            FOREIGN KEY (asset_id) REFERENCES assets(id)
        );

        CREATE TABLE IF NOT EXISTS votes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            proposal_id INTEGER NOT NULL,
            voter_address TEXT NOT NULL,
            vote_type BOOLEAN NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (proposal_id) REFERENCES proposals(id),
            UNIQUE(proposal_id, voter_address)
        );

        CREATE TABLE IF NOT EXISTS balances (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            asset_id INTEGER NOT NULL,
            holder_address TEXT NOT NULL,
            amount INTEGER NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (asset_id) REFERENCES assets(id),
            UNIQUE(asset_id, holder_address)
        );
        "#
    )
    .execute(pool)
    .await
    .context("Failed to create tables with foreign keys")?;

    // Create indices
    sqlx::query!(
        r#"
        CREATE INDEX IF NOT EXISTS idx_funds_version ON funds(version);
        CREATE INDEX IF NOT EXISTS idx_assets_version ON assets(version);
        CREATE INDEX IF NOT EXISTS idx_fund_members_status ON fund_members(status);
        CREATE INDEX IF NOT EXISTS idx_assets_address ON assets(address);
        CREATE INDEX IF NOT EXISTS idx_fund_members_fund_id ON fund_members(fund_id);
        CREATE INDEX IF NOT EXISTS idx_fund_wallets_fund_id ON fund_wallets(fund_id);
        CREATE INDEX IF NOT EXISTS idx_investments_fund_id ON investments(fund_id);
        CREATE INDEX IF NOT EXISTS idx_investments_asset_id ON investments(asset_id);
        CREATE INDEX IF NOT EXISTS idx_messages_fund_id ON messages(fund_id, id DESC);
        CREATE INDEX IF NOT EXISTS idx_positions_fund_id ON positions(fund_id);
        CREATE INDEX IF NOT EXISTS idx_votes_proposal_id ON votes(proposal_id);
        CREATE INDEX IF NOT EXISTS idx_balances_asset_id ON balances(asset_id);
        "#
    )
    .execute(pool)
    .await
    .context("Failed to create indices")?;

    Ok(())
} 
