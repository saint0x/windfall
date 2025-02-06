use actix_web::{web, HttpResponse, Responder};
use serde::{Serialize};
use aptos_sdk::types::account_address::AccountAddress;
use crate::{
    client::Client,
    error::{AppError, Result},
};

#[derive(Serialize)]
pub struct BalanceResponse {
    pub address: String,
    pub balance: u64,
}

#[derive(Serialize)]
pub struct ModulesResponse {
    pub address: String,
    pub modules: Vec<String>,
}

#[derive(Serialize)]
pub struct ResourcesResponse {
    pub address: String,
    pub resources: Vec<serde_json::Value>,
}

#[derive(Clone)]
pub struct AccountClient<'a> {
    client: &'a Client,
}

impl<'a> AccountClient<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn get_balance(&self, address: AccountAddress) -> Result<u64> {
        self.client.get_account_balance(address).await
    }

    pub async fn get_sequence_number(&self, address: AccountAddress) -> Result<u64> {
        self.client.get_sequence_number(address).await
    }
}

pub async fn get_balance(
    client: web::Data<Client>,
    address: web::Path<String>,
) -> impl Responder {
    let address = match AccountAddress::from_hex_literal(&address) {
        Ok(addr) => addr,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid address format"
        })),
    };

    match client.get_account_balance(address).await {
        Ok(balance) => HttpResponse::Ok().json(BalanceResponse {
            address: format!("0x{}", address),
            balance,
        }),
        Err(AppError::NotFound(_)) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Account not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn get_modules(
    client: web::Data<Client>,
    address: web::Path<String>,
) -> impl Responder {
    if address.as_str() == "0x1" {
        match client.get_core_account_modules().await {
            Ok(modules) => HttpResponse::Ok().json(ModulesResponse {
                address: "0x1".to_string(),
                modules,
            }),
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    } else {
        let _address = match AccountAddress::from_hex_literal(&address) {
            Ok(addr) => addr,
            Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid address format"
            })),
        };

        // TODO: Implement get_account_modules for non-core addresses
        HttpResponse::NotImplemented().json(serde_json::json!({
            "error": "Getting modules for non-core addresses is not yet implemented"
        }))
    }
}

pub async fn get_resources(
    _client: web::Data<Client>,
    address: web::Path<String>,
) -> impl Responder {
    let _address = match AccountAddress::from_hex_literal(&address) {
        Ok(addr) => addr,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid address format"
        })),
    };

    // TODO: Implement get_account_resources
    HttpResponse::NotImplemented().json(serde_json::json!({
        "error": "Getting account resources is not yet implemented"
    }))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/accounts")
            .route("/{address}/balance", web::get().to(get_balance))
            .route("/{address}/modules", web::get().to(get_modules))
            .route("/{address}/resources", web::get().to(get_resources))
    );
} 