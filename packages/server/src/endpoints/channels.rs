use actix_web::{error::Error, get, post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{query, query_as, query_as_unchecked};
use validator::Validate;

use crate::consts::{get_channel_access_data, send_to_server_members};
use crate::errors::RouteError;
use crate::id_type::OptionId;
use crate::structures::session::Session;
use crate::ws::JsonMessage;
use crate::{
    errors::{self, ErrorResponse},
    structures::{self},
    AppState,
};

#[get("/servers/{id}/channels")]
async fn server_channels(
    data: web::Data<AppState>,
    path: web::Path<u64>,
    session: web::ReqData<Session>,
) -> Result<impl Responder, Error> {
    let server_id = path.into_inner();

    let member = query!(
        "SELECT id FROM Member WHERE user_id = ? AND server_id = ?",
        session.user_id,
        server_id
    )
    .fetch_optional(&data.db)
    .await
    .map_err(errors::Errors::Db)?;

    if member.is_none() {
        return Ok(HttpResponse::NotFound().finish());
    }

    let channels = query_as_unchecked!(
        structures::channel::Channel,
        "SELECT id, created_at, name, kind AS `kind: _`, server_id FROM Channel WHERE server_id = ? ORDER BY id",
        server_id
    )
        .fetch_all(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    Ok(HttpResponse::Ok().json(channels))
}

#[derive(Deserialize, Validate)]
pub struct CreateChannelData {
    #[validate(length(min = 2, max = 32))]
    name: String,
}

#[post("/servers/{id}/channels")]
async fn create_channel(
    data: web::Data<AppState>,
    path: web::Path<u64>,
    req_body: web::Either<web::Json<CreateChannelData>, web::Form<CreateChannelData>>,
    session: web::ReqData<Session>,
) -> Result<impl Responder, Error> {
    let server_id = path.into_inner();

    let member = query!(
        "SELECT id FROM Member WHERE user_id = ? AND server_id = ?",
        session.user_id,
        server_id
    )
    .fetch_optional(&data.db)
    .await
    .map_err(errors::Errors::Db)?;

    if member.is_none() {
        return Ok(HttpResponse::NotFound().finish());
    }

    // TODO: improve this with a permission system
    let server_owner = query!("SELECT owner_id FROM Server WHERE id = ?", server_id)
        .fetch_one(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    if server_owner.owner_id != session.user_id.0 {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let channels = query!(
        "SELECT COUNT(*) AS count FROM Channel WHERE server_id = ?",
        server_id
    )
    .fetch_one(&data.db)
    .await
    .map_err(errors::Errors::Db)?;

    if channels.count >= 200 {
        return Ok(HttpResponse::TooManyRequests().json(ErrorResponse {
            errors: "Channel limit reached. Max: 200".to_string(),
        }));
    }

    let create_data = req_body.into_inner();

    create_data.validate().map_err(errors::Errors::Validation)?;

    let channel: (u64, DateTime<Utc>) =
        query_as("INSERT INTO Channel VALUES (NULL, DEFAULT, ?, ?, ?) RETURNING id, created_at")
            .bind(create_data.name.clone())
            .bind(structures::channel::ChannelKind::Text)
            .bind(server_id)
            .fetch_one(&data.db)
            .await
            .map_err(errors::Errors::Db)?;

    let channel_struct = structures::channel::Channel {
        id: channel.0.into(),
        created_at: channel.1,
        name: create_data.name,
        kind: structures::channel::ChannelKind::Text,
        server_id: OptionId(Some(server_id)),
    };

    send_to_server_members(
        &data,
        server_id,
        &JsonMessage(serde_json::json!({
            "type": "channel_create",
            "data": channel_struct.clone()
        })),
    );

    Ok(HttpResponse::Ok().json(channel_struct))
}

#[get("/channels/{channel_id}")]
async fn server_channel(
    data: web::Data<AppState>,
    path: web::Path<u64>,
    session: web::ReqData<Session>,
) -> Result<impl Responder, RouteError> {
    let channel_id = path.into_inner();

    Ok(HttpResponse::Ok().json(
        get_channel_access_data(&data, channel_id, session.user_id.0)
            .await?
            .0,
    ))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(create_channel)
        .service(server_channels)
        .service(server_channel);
}
