pub mod funds;
pub mod members;
pub mod messages;
pub mod proposals;
pub mod assets;
pub mod transactions;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(funds::scope())
       .service(members::scope())
       .service(messages::scope())
       .service(proposals::scope())
       .service(assets::scope())
       .service(transactions::scope());
} 