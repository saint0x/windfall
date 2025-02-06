use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use crate::AppState;
use aptos_sdk::rest_client::Transaction;

#[derive(Serialize)]
pub struct TransactionStatus {
    hash: String,
    status: String,
    success: bool,
    version: Option<u64>,
    vm_status: Option<String>,
    gas_used: Option<u64>,
}

pub fn scope() -> actix_web::Scope {
    web::scope("/transactions")
        .service(get_transaction_status)
}

#[get("/{hash}")]
async fn get_transaction_status(
    state: web::Data<AppState>,
    hash: web::Path<String>,
) -> impl Responder {
    match state.client.get_transaction_status(&hash).await {
        Ok(txn) => {
            let status = TransactionStatus {
                hash: hash.to_string(),
                status: txn.type_str().to_string(),
                success: txn.success(),
                version: txn.version(),
                vm_status: Some(txn.vm_status().to_string()),
                gas_used: match txn {
                    Transaction::UserTransaction(t) => Some(t.info.gas_used.0),
                    _ => None,
                },
            };
            HttpResponse::Ok().json(status)
        },
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
} 