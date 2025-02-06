use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::db::operations;

#[derive(Deserialize)]
pub struct CreateFundWalletRequest {
    actuator_address: String,
    members: Vec<MemberInput>,
}

#[derive(Deserialize)]
pub struct MemberInput {
    address: String,
    ownership_share: u64,  // Basis points (1/10000)
}

#[derive(Deserialize)]
pub struct InvestmentRequest {
    target_address: String,
    amount: u64,
    asset_id: i64,
}

#[derive(Deserialize)]
pub struct WithdrawRequest {
    amount: u64,
}

#[derive(Deserialize)]
pub struct UpdateShareRequest {
    new_share: u64,
}

#[derive(Serialize)]
pub struct FundWalletResponse {
    fund_id: i64,
    actuator_address: String,
    balance: u64,
    members: Vec<MemberInfo>,
}

#[derive(Serialize)]
pub struct MemberInfo {
    address: String,
    ownership_share: u64,
    joined_at: i64,
}

pub fn scope() -> actix_web::Scope {
    web::scope("/funds/{fund_id}/wallet")
        .service(create_fund_wallet)
        .service(get_fund_wallet)
        .service(invest)
        .service(withdraw_profits)
        .service(update_member_share)
}

#[post("")]
async fn create_fund_wallet(
    state: web::Data<AppState>,
    fund_id: web::Path<i64>,
    req: web::Json<CreateFundWalletRequest>,
) -> impl Responder {
    // Validate total shares = 10000 (100%)
    let total_shares: u64 = req.members.iter().map(|m| m.ownership_share).sum();
    if total_shares != 10000 {
        return HttpResponse::BadRequest().body("Total ownership shares must equal 10000 (100%)");
    }

    // First create the fund wallet
    let wallet = match operations::create_fund_wallet(
        &state.db,
        fund_id.into_inner(),
        &req.actuator_address,
    ).await {
        Ok(w) => w,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    // Then add all members with their shares
    for member in &req.members {
        match operations::create_fund_member(
            &state.db,
            wallet.fund_id,
            &member.address,
            member.ownership_share as i64,
        ).await {
            Ok(_) => (),
            Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to add member {}: {}", member.address, e)),
        }
    }

    HttpResponse::Ok().json(wallet)
}

#[get("")]
async fn get_fund_wallet(
    state: web::Data<AppState>,
    fund_id: web::Path<i64>,
) -> impl Responder {
    match operations::get_fund_wallet(&state.db, fund_id.into_inner()).await {
        Ok(wallet) => HttpResponse::Ok().json(wallet),
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
}

#[post("/invest")]
async fn invest(
    state: web::Data<AppState>,
    fund_id: web::Path<i64>,
    req: web::Json<InvestmentRequest>,
) -> impl Responder {
    let amount: i64 = match req.amount.try_into() {
        Ok(val) => val,
        Err(_) => return HttpResponse::BadRequest().body("Amount too large"),
    };

    match operations::create_investment(
        &state.db,
        fund_id.into_inner(),
        req.asset_id,
        amount,
        &req.target_address,
    ).await {
        Ok(investment) => HttpResponse::Ok().json(investment),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/withdraw")]
async fn withdraw_profits(
    state: web::Data<AppState>,
    fund_id: web::Path<i64>,
    req: web::Json<WithdrawRequest>,
) -> impl Responder {
    let amount: i64 = match req.amount.try_into() {
        Ok(val) => val,
        Err(_) => return HttpResponse::BadRequest().body("Amount too large"),
    };

    match operations::withdraw_investment(
        &state.db,
        fund_id.into_inner(),
        amount,
    ).await {
        Ok(withdrawal) => HttpResponse::Ok().json(withdrawal),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/members/{member_address}/share")]
async fn update_member_share(
    state: web::Data<AppState>,
    path: web::Path<(i64, String)>,
    req: web::Json<UpdateShareRequest>,
) -> impl Responder {
    let (fund_id, member_address) = path.into_inner();
    let new_share: i64 = match req.new_share.try_into() {
        Ok(val) => val,
        Err(_) => return HttpResponse::BadRequest().body("Share value too large"),
    };
    
    match operations::update_member_share(
        &state.db,
        fund_id,
        &member_address,
        new_share,
    ).await {
        Ok(member) => HttpResponse::Ok().json(member),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
} 