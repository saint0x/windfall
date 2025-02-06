use actix_web::{web, HttpResponse, Result};
use aptos_sdk::types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};
use crate::api::ApiState;
use crate::error::ClientError;

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    address: String,
    balance: u64,
}

#[derive(Debug, Serialize)]
pub struct ResourcesResponse {
    address: String,
    resources: Vec<ResourceData>,
}

#[derive(Debug, Serialize)]
pub struct ResourceData {
    type_: String,
    data: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ModulesResponse {
    address: String,
    modules: Vec<String>,
}

pub async fn get_balance(
    state: web::Data<ApiState>,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let address = path.into_inner();
    let account_address = AccountAddress::from_hex_literal(&address)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid address: {}", e)))?;

    let balance = state
        .client
        .get_account_balance(account_address)
        .await
        .map_err(|e| match e {
            ClientError::ResourceNotFound(_) => actix_web::error::ErrorNotFound(e.to_string()),
            _ => actix_web::error::ErrorInternalServerError(e.to_string()),
        })?;

    Ok(HttpResponse::Ok().json(BalanceResponse {
        address,
        balance,
    }))
}

pub async fn get_resources(
    state: web::Data<ApiState>,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let address = path.into_inner();
    let account_address = AccountAddress::from_hex_literal(&address)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid address: {}", e)))?;

    let resources = state
        .client
        .get_account_resources(account_address)
        .await
        .map_err(|e| match e {
            ClientError::ResourceNotFound(_) => actix_web::error::ErrorNotFound(e.to_string()),
            _ => actix_web::error::ErrorInternalServerError(e.to_string()),
        })?;

    Ok(HttpResponse::Ok().json(ResourcesResponse {
        address,
        resources: resources
            .into_iter()
            .map(|r| ResourceData {
                type_: r.type_,
                data: r.data,
            })
            .collect(),
    }))
}

pub async fn get_modules(
    state: web::Data<ApiState>,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let address = path.into_inner();
    let account_address = AccountAddress::from_hex_literal(&address)
        .map_err(|e| actix_web::error::ErrorBadRequest(format!("Invalid address: {}", e)))?;

    let modules = state
        .client
        .get_account_modules(account_address)
        .await
        .map_err(|e| match e {
            ClientError::ResourceNotFound(_) => actix_web::error::ErrorNotFound(e.to_string()),
            _ => actix_web::error::ErrorInternalServerError(e.to_string()),
        })?;

    Ok(HttpResponse::Ok().json(ModulesResponse {
        address,
        modules,
    }))
} 