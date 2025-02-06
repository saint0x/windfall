pub mod client;
pub mod db;
pub mod models;

use backend::{
    AppState,
    db::{operations, schema::*},
    error::Result,
    Client,
};
use sqlx::sqlite::SqlitePool;
use crate::{setup_test_db, AptosRestClientInterface};
use mockall::Mock;
use aptos_sdk::{
    types::{
        account_address::AccountAddress,
        transaction::{SignedTransaction, TransactionPayload},
    },
}; 