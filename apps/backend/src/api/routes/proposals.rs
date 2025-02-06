use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use chrono::{DateTime, Utc};
use log::error;
use crate::{
    AppState,
    db::{operations, types::DbDateTime},
};

#[derive(Deserialize)]
pub struct CreateProposalRequest {
    title: String,
    description: String,
    proposer_address: String,
    end_time: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct VoteRequest {
    voter_address: String,
    vote_type: bool,
}

#[derive(Deserialize)]
pub struct EmergencyVetoRequest {
    initiator_address: String,
}

pub fn scope() -> actix_web::Scope {
    web::scope("/funds/{fund_id}/proposals")
        .service(create_proposal)
        .service(get_proposal)
        .service(vote_on_proposal)
        .service(emergency_veto)
}

#[post("")]
async fn create_proposal(
    state: web::Data<AppState>,
    _fund_id: web::Path<i64>,
    req: web::Json<CreateProposalRequest>,
) -> impl Responder {
    // First create the proposal
    let proposal = match operations::create_proposal(
        &state.db,
        &req.title,
        &req.description,
        DbDateTime::from(req.end_time),
    ).await {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    // Then sync it with the proposer address
    match operations::sync_proposal_creation(
        &state.db,
        proposal.id as u64,
        &req.proposer_address,
    ).await {
        Ok(proposal) => HttpResponse::Ok().json(proposal),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/{proposal_id}")]
async fn get_proposal(
    state: web::Data<AppState>,
    proposal_id: web::Path<i64>,
) -> impl Responder {
    match operations::get_proposal(&state.db, proposal_id.into_inner()).await {
        Ok(proposal) => HttpResponse::Ok().json(proposal),
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
}

#[post("/{proposal_id}/votes")]
async fn vote_on_proposal(
    state: web::Data<AppState>,
    proposal_id: web::Path<i64>,
    req: web::Json<VoteRequest>,
) -> impl Responder {
    let voter = match req.voter_address.parse() {
        Ok(addr) => addr,
        Err(_) => return HttpResponse::BadRequest().body("Invalid voter address"),
    };

    match operations::vote_on_proposal(
        &state.db,
        proposal_id.into_inner(),
        voter,
        req.vote_type,
    ).await {
        Ok(vote) => HttpResponse::Ok().json(vote),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[post("/{proposal_id}/emergency-veto")]
async fn emergency_veto(
    state: web::Data<AppState>,
    proposal_id: web::Path<i64>,
    req: web::Json<EmergencyVetoRequest>,
) -> impl Responder {
    // TODO: Add authorization check for initiator_address
    // For now, just log who initiated the veto
    log::info!("Emergency veto initiated by: {}", req.initiator_address);

    match operations::emergency_veto_proposal(
        &state.db,
        proposal_id.into_inner(),
    )
    .await
    {
        Ok(proposal) => HttpResponse::Ok().json(proposal),
        Err(e) => {
            error!("Failed to veto proposal: {}", e);
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
} 