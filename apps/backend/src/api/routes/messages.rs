use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use crate::AppState;
use crate::db::operations;

#[derive(Deserialize)]
pub struct CreateMessageRequest {
    pub content: String,
    pub sender_address: String,
}

#[derive(Deserialize)]
pub struct GetMessagesQuery {
    pub limit: Option<i64>,
    pub before_id: Option<i64>,
}

pub fn scope() -> actix_web::Scope {
    web::scope("/funds/{fund_id}/messages")
        .service(create_message)
        .service(get_messages)
}

#[post("")]
async fn create_message(
    state: web::Data<AppState>,
    fund_id: web::Path<i64>,
    req: web::Json<CreateMessageRequest>,
) -> impl Responder {
    let sender = match req.sender_address.parse() {
        Ok(addr) => addr,
        Err(_) => return HttpResponse::BadRequest().body("Invalid sender address"),
    };

    match operations::create_message(
        &state.db,
        fund_id.into_inner(),
        sender,
        req.content.clone(),
    ).await {
        Ok(message) => HttpResponse::Ok().json(message),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("")]
async fn get_messages(
    state: web::Data<AppState>,
    fund_id: web::Path<i64>,
    query: web::Query<GetMessagesQuery>,
) -> impl Responder {
    match operations::get_messages(
        &state.db,
        fund_id.into_inner(),
        query.limit.unwrap_or(50),
        query.before_id,
    ).await {
        Ok(messages) => HttpResponse::Ok().json(messages),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
} 