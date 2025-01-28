use sqlx::sqlite::{SqlitePool, SqliteRow};
use sqlx::{FromRow, Row};
use aptos_sdk::types::account_address::AccountAddress;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Fund {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub executor: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct FundMetadata {
    pub id: i64,
    pub fund_id: i64,
    pub key: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct FundMember {
    pub id: i64,
    pub fund_id: i64,
    pub member_address: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub fund_id: i64,
    pub sender: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Position {
    pub id: i64,
    pub fund_id: i64,
    pub size: i64,
    pub price: i64,
    pub is_long: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Proposal {
    pub id: i64,
    pub fund_id: i64,
    pub proposal_type: i64,
    pub votes_yes: i64,
    pub votes_no: i64,
    pub end_time: DateTime<Utc>,
    pub executed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Vote {
    pub id: i64,
    pub proposal_id: i64,
    pub voter: String,
    pub vote_yes: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Asset {
    pub id: i64,
    pub symbol: String,
    pub name: String,
    pub decimals: i64,
    pub total_supply: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Balance {
    pub id: i64,
    pub asset_id: i64,
    pub holder: String,
    pub amount: i64,
    pub last_updated: DateTime<Utc>,
}

impl Fund {
    pub fn executor_address(&self) -> AccountAddress {
        AccountAddress::from_str(&self.executor).unwrap()
    }
}

impl FundMember {
    pub fn member_address(&self) -> AccountAddress {
        AccountAddress::from_str(&self.member_address).unwrap()
    }
}

impl Message {
    pub fn sender_address(&self) -> AccountAddress {
        AccountAddress::from_str(&self.sender).unwrap()
    }
}

impl Vote {
    pub fn voter_address(&self) -> AccountAddress {
        AccountAddress::from_str(&self.voter).unwrap()
    }
}

impl Balance {
    pub fn holder_address(&self) -> AccountAddress {
        AccountAddress::from_str(&self.holder).unwrap()
    }
} 
