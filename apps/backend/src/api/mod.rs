pub mod account;
pub mod routes;
pub mod events;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    account::configure(cfg);
} 