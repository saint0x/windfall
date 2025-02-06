use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use crate::AppState;
use crate::db::operations;

#[derive(Deserialize)]
pub struct AddMemberRequest {
    pub member_address: String,
}

pub fn scope() -> actix_web::Scope {
    web::scope("/funds/{fund_id}/members")
        .service(add_member)
}

#[post("")]
async fn add_member(
    state: web::Data<AppState>,
    fund_id: web::Path<i64>,
    req: web::Json<AddMemberRequest>,
) -> impl Responder {
    let member = match req.member_address.parse() {
        Ok(addr) => addr,
        Err(_) => return HttpResponse::BadRequest().body("Invalid member address"),
    };

    match operations::add_fund_member(&state.db, fund_id.into_inner(), member).await {
        Ok(member) => HttpResponse::Ok().json(member),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
} 