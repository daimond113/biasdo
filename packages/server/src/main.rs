mod consts;
mod endpoints;
mod errors;
mod id_type;
mod structures;
mod ws;

use crate::ws::MyWebSocket;
use crate::{consts::SESSION_COOKIE_NAME, errors::Errors};
use actix::Addr;
use actix_cors::Cors;
use actix_web::{
    dev::ServiceRequest,
    error::ErrorUnauthorized,
    middleware::{Compress, Logger},
    rt, web, App, Error, HttpMessage, HttpServer,
};
use actix_web_httpauth::{extractors::bearer::BearerAuth, middleware::HttpAuthentication};
use dashmap::{DashMap, DashSet};
use sqlx::{mysql::MySqlPoolOptions, query, query_as, MySqlPool};
use std::time::Duration;

#[derive(Debug)]
pub struct AppState {
    pub db: MySqlPool,
    // server id -> user id(s)
    pub server_connections: DashMap<u64, DashSet<u64>>,
    // user id -> ws(s) // multiple sessions
    pub user_connections: DashMap<u64, DashSet<Addr<MyWebSocket>>>,
}

pub async fn validator(
    req: ServiceRequest,
    _credentials: Option<BearerAuth>,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let app_data = req.app_data::<web::Data<AppState>>().unwrap();

    if let Some(cookie) = req.cookie(SESSION_COOKIE_NAME) {
        let value = cookie.value();

        return match query_as!(
            structures::session::Session,
            "SELECT id, created_at, expires_at, user_id FROM Session WHERE id = ?",
            value
        )
        .fetch_optional(&app_data.db)
        .await
        {
            Ok(Some(session)) => {
                req.extensions_mut().insert(session);
                Ok(req)
            }
            Ok(None) => Err((ErrorUnauthorized("unauthorized"), req)),
            Err(e) => Err((Errors::Db(e).into(), req)),
        };
    };

    Err((ErrorUnauthorized("unauthorized"), req))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let address = std::env::var("ADDRESS").unwrap_or("127.0.0.1".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .expect("Invalid port");
    let db_url = &std::env::var("DATABASE_URL").expect("No database url");

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await
        .expect("Error connecting to database");

    println!("Listening on {}:{}", address, port);

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("error"));

    let data = web::Data::new(AppState {
        db: pool.clone(),
        server_connections: DashMap::new(),
        user_connections: DashMap::new(),
    });

    let db = pool.clone();

    rt::spawn(async move {
        let mut inter = rt::time::interval(Duration::from_secs(60));
        loop {
            inter.tick().await;

            query!("DELETE FROM Session WHERE expires_at <= NOW()")
                .execute(&db)
                .await
                .ok();

            query!("DELETE FROM Invite WHERE expires_at <= NOW()")
                .execute(&db)
                .await
                .ok();
        }
    });

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(Compress::default())
            .wrap(Logger::default())
            .app_data(web::Data::clone(&data))
            .service(endpoints::auth::auth)
            .service(
                web::scope("/v0")
                    .wrap(HttpAuthentication::with_fn(validator))
                    .configure(endpoints::configure),
            )
    })
    .bind((address, port))?
    .run()
    .await
}
