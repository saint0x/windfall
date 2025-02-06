use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use crate::AppState;
use crate::db::operations;

#[derive(Deserialize)]
pub struct CreateFundRequest {
    pub name: String,
    pub executor_address: String,
}

pub fn scope() -> actix_web::Scope {
    web::scope("/funds")
        .service(create_fund)
        .service(get_fund)
        .service(get_fund_members)
}

#[post("")]
async fn create_fund(
    state: web::Data<AppState>,
    req: web::Json<CreateFundRequest>,
) -> impl Responder {
    match operations::create_fund(&state.db, req.name.clone(), req.executor_address.clone()).await {
        Ok(fund) => HttpResponse::Ok().json(fund),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/{fund_id}")]
async fn get_fund(
    state: web::Data<AppState>,
    fund_id: web::Path<i64>,
) -> impl Responder {
    match operations::get_fund(&state.db, fund_id.into_inner()).await {
        Ok(fund) => HttpResponse::Ok().json(fund),
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
}

#[get("/{fund_id}/members")]
async fn get_fund_members(
    state: web::Data<AppState>,
    fund_id: web::Path<i64>,
) -> impl Responder {
    match operations::get_fund_members(&state.db, fund_id.into_inner()).await {
        Ok(members) => HttpResponse::Ok().json(members),
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
} 