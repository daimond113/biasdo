use actix_web::{error::Error, get, post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{query, query_as};

use crate::consts::{merge_json, send_to_server_members};
use crate::ws::JsonMessage;
use crate::{
    errors, get_member_opt_user_value, get_member_value,
    structures::{self, session::Session},
    AppState,
};

#[derive(Deserialize, Debug)]
pub struct ServerMembersQuery {
    last_id: Option<u64>,
}

struct MemberResult {
    member_id: u64,
    member_created_at: DateTime<Utc>,
    member_server_id: u64,
    member_user_id: Option<u64>,
    member_nickname: Option<String>,
    user_created_at: Option<DateTime<Utc>>,
    user_username: Option<String>,
}

#[get("/servers/{server_id}/members")]
async fn server_members(
    data: web::Data<AppState>,
    path: web::Path<u64>,
    query: Option<web::Query<ServerMembersQuery>>,
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

    let last_id_opt = query.map(|q| q.into_inner().last_id).unwrap_or(None);

    let mut members = match last_id_opt {
        Some(last_id) => query_as!(
            MemberResult,
            "SELECT Member.id AS member_id, Member.created_at AS member_created_at, Member.server_id AS member_server_id, Member.user_id AS member_user_id, Member.nickname AS member_nickname, User.created_at AS user_created_at, User.username AS user_username FROM Member LEFT JOIN User ON User.id = Member.user_id WHERE Member.server_id = ? AND Member.id < ? ORDER BY Member.id LIMIT 100",
            server_id,
            last_id
        )
            .fetch_all(&data.db)
            .await
            .map_err(errors::Errors::Db)?,
        None => query_as!(
            MemberResult,
            "SELECT Member.id AS member_id, Member.created_at AS member_created_at, Member.server_id AS member_server_id, Member.user_id AS member_user_id, Member.nickname AS member_nickname, User.created_at AS user_created_at, User.username AS user_username FROM Member LEFT JOIN User ON User.id = Member.user_id WHERE Member.server_id = ? ORDER BY Member.id LIMIT 100",
            server_id
        )
            .fetch_all(&data.db)
            .await
            .map_err(errors::Errors::Db)?
    };

    members.reverse();

    Ok(HttpResponse::Ok().json(
        members
            .iter()
            .map(|r| get_member_opt_user_value!(r))
            .collect::<serde_json::Value>(),
    ))
}

#[get("/servers/{server_id}/members/{member_id}")]
async fn server_member(
    data: web::Data<AppState>,
    path: web::Path<(u64, u64)>,
) -> Result<impl Responder, Error> {
    let (server_id, member_id) = path.into_inner();

    let record_opt = query!(
        "SELECT Member.id AS member_id, Member.created_at AS member_created_at, Member.server_id AS member_server_id, Member.user_id AS member_user_id, Member.nickname AS member_nickname, User.created_at AS user_created_at, User.username AS user_username FROM Member INNER JOIN User ON User.id = Member.user_id WHERE Member.server_id = ? AND Member.id = ? LIMIT 100",
        server_id,
        member_id
    )
        .fetch_optional(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    if record_opt.is_none() {
        return Ok(HttpResponse::NotFound().finish());
    }

    let record = record_opt.unwrap();

    Ok(HttpResponse::Ok().json(get_member_value!(record)))
}

#[post("/servers/{server_id}/leave")]
async fn leave_server(
    data: web::Data<AppState>,
    path: web::Path<u64>,
    session: web::ReqData<Session>,
) -> Result<impl Responder, Error> {
    let server_id = path.into_inner();

    let record_opt = query!(
        "SELECT Member.id AS member_id, Member.created_at AS member_created_at, Member.server_id AS member_server_id, Member.user_id AS member_user_id, Member.nickname AS member_nickname, User.created_at AS user_created_at, User.username AS user_username FROM Member INNER JOIN User ON User.id = Member.user_id WHERE Member.server_id = ? AND Member.user_id = ?",
        server_id,
        session.user_id,
    )
        .fetch_optional(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    if record_opt.is_none() {
        return Ok(HttpResponse::NotFound().finish());
    }

    let record = record_opt.unwrap();

    query!(
        "UPDATE Member SET user_id=NULL, nickname=NULL WHERE id = ? AND server_id = ?",
        record.member_id,
        server_id
    )
    .execute(&data.db)
    .await
    .map_err(errors::Errors::Db)?;

    let member_value = get_member_value!(record);

    send_to_server_members(
        &data,
        server_id,
        &JsonMessage(serde_json::json!({
            "type": "member_delete",
            "data": member_value.clone()
        })),
    );

    let server = query_as!(
        structures::server::Server,
        "SELECT id, created_at, name, owner_id FROM Server WHERE id = ?",
        server_id
    )
    .fetch_one(&data.db)
    .await
    .map_err(errors::Errors::Db)?;

    let server_msg = JsonMessage(serde_json::json!({
        "type": "server_delete",
        "data": server.clone()
    }));

    if let Some(user_connections) = data.user_connections.get(&session.user_id.0) {
        user_connections.iter().for_each(|addr| {
            addr.do_send(server_msg.clone());
        });

        data.server_connections.entry(server_id).and_modify(|map| {
            map.remove(&session.user_id.0);
        });
    }

    Ok(HttpResponse::Ok().json(member_value))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(server_members)
        .service(server_member)
        .service(leave_server);
}
