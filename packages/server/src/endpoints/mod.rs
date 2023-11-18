use actix_web::web;

pub mod auth;
pub mod channels;
pub mod invites;
pub mod members;
pub mod messages;
pub mod servers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(auth::configure)
        .configure(channels::configure)
        .configure(invites::configure)
        .configure(members::configure)
        .configure(messages::configure)
        .configure(servers::configure);
}
