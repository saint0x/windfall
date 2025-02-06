use aptos_sdk::{
    types::{
        account_address::AccountAddress,
    },
};

pub mod api;
pub mod client;
pub mod db;
pub mod error;
pub mod utils;
pub mod config;
pub mod sync;

// Re-export commonly used types
pub use aptos_sdk::types as aptos_types;
pub use error::{AppError, Result};
pub use client::Client;
pub use config::ClientConfig;
pub use db::{create_pool, Pool};
pub use utils::*;
pub use sync::BlockchainSynchronizer;

#[derive(Debug)]
pub struct SigningAccount {
    pub address: AccountAddress,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Pool,
    pub client: Client,
}
