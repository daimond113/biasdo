use crate::{
    consts::merge_json,
    errors::{self},
    structures::{self, session::Session},
    AppState,
};
use actix_web::{error::Error, get, post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use cuid2::create_id;
use sqlx::{query, query_as};
use crate::consts::send_to_server_members;
use crate::ws::JsonMessage;

#[get("/invites/{id}")]
async fn get_invite(
    data: web::Data<AppState>,
    path: web::Path<String>,
    session: web::ReqData<Session>,
) -> Result<impl Responder, Error> {
    let invite_id = path.into_inner();

    let invite = query!(
        "SELECT Invite.id, Invite.created_at, Invite.expires_at, Invite.server_id, Server.created_at AS server_created_at, Server.name AS server_name, Server.owner_id AS server_owner_id FROM Invite INNER JOIN Server ON Server.id = Invite.server_id WHERE Invite.id = ?",
        invite_id
    )
        .fetch_optional(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    match invite {
        Some(record) => {
            let member = query!(
                "SELECT id FROM Member WHERE user_id = ? AND server_id = ?",
                session.user_id,
                record.server_id
            )
                .fetch_optional(&data.db)
                .await
                .map_err(errors::Errors::Db)?;

            if member.is_some() {
                return Ok(HttpResponse::Conflict().json(serde_json::json!({
                    "errors": "You are already a member of this server.",
                    "server_id": record.server_id,
                })));
            }

            let invite = structures::invite::Invite {
                id: record.id,
                created_at: record.created_at,
                expires_at: record.expires_at,
                server_id: record.server_id.into(),
            };

            let mut invite_value = serde_json::to_value(invite.clone()).unwrap();

            let server = structures::server::Server {
                id: record.server_id.into(),
                created_at: record.server_created_at,
                name: record.server_name,
                owner_id: record.server_owner_id.into(),
            };

            merge_json(&mut invite_value, &serde_json::json!({ "server": server }));

            Ok(HttpResponse::Ok().json(invite_value))
        }
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

#[get("/servers/{id}/invites")]
async fn get_invites(
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

    let record = query_as!(
        structures::invite::Invite,
        "SELECT id, created_at, expires_at, server_id FROM Invite WHERE server_id = ? ORDER BY created_at DESC",
        server_id
    )
        .fetch_all(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    Ok(HttpResponse::Ok().json(record))
}

#[post("/servers/{id}/invites")]
async fn create_invite(
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

    let invite_id = create_id();

    let invite: (DateTime<Utc>, DateTime<Utc>) = query_as(
        "INSERT INTO Invite VALUES (?, DEFAULT, DEFAULT, ?) RETURNING created_at, expires_at"
    )
        .bind(invite_id.clone())
        .bind(server_id)
        .fetch_one(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    let invite_struct = structures::invite::Invite {
        id: invite_id,
        created_at: invite.0,
        expires_at: invite.1,
        server_id: server_id.into(),
    };

    send_to_server_members(
        &data,
        server_id,
        &JsonMessage(serde_json::json!({
            "type": "invite_create",
            "data": invite_struct.clone()
        }))
    );

    Ok(HttpResponse::Ok().json(invite_struct))
}

#[post("/invites/{id}/join")]
async fn join_invite(
    data: web::Data<AppState>,
    path: web::Path<String>,
    session: web::ReqData<Session>,
) -> Result<impl Responder, Error> {
    let invite_id = path.into_inner();

    let invite = query_as!(
        structures::invite::Invite,
        "SELECT id, created_at, expires_at, server_id FROM Invite WHERE id = ?",
        invite_id
    )
        .fetch_optional(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    if invite.is_none() {
        return Ok(HttpResponse::NotFound().finish());
    }

    let invite = invite.unwrap();

    let existing_member = query!(
        "SELECT id FROM Member WHERE user_id = ? AND server_id = ?",
        session.user_id,
        invite.server_id
    )
        .fetch_optional(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    if existing_member.is_some() {
        return Ok(HttpResponse::Conflict().json(serde_json::json!({
            "errors": "You are already a member of this server.",
            "server_id": invite.server_id,
        })));
    }

    let member: (u64, DateTime<Utc>) = query_as(
        "INSERT INTO Member VALUES (NULL, DEFAULT, ?, ?, NULL) RETURNING id, created_at"
    )
        .bind(invite.server_id)
        .bind(session.user_id)
        .fetch_one(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    let user = query_as!(
        structures::user::User,
        "SELECT id, created_at, username, password FROM User WHERE id = ?",
        session.user_id
    )
        .fetch_one(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    let member_struct = structures::member::Member {
        id: member.0.into(),
        created_at: member.1,
        server_id: invite.server_id.into(),
        user_id: Some(session.user_id.0).into(),
        nickname: None,
    };

    let mut member_value = serde_json::to_value(member_struct.clone()).unwrap();

    merge_json(&mut member_value, &serde_json::json!({ "user": user }));

    send_to_server_members(
        &data,
        invite.server_id.0,
        &JsonMessage(serde_json::json!({
            "type": "member_create",
            "data": member_value.clone()
        }))
    );

    Ok(HttpResponse::Ok().json(member_value))
}