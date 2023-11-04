use actix_web::{error::Error, get, web, HttpResponse, Responder, post};
use serde::Deserialize;
use sqlx::{query, query_as};

use crate::{
    errors,
    structures::{
        self,
        session::Session,
    },
    AppState,
};
use crate::consts::{merge_json, send_to_server_members};
use crate::ws::JsonMessage;

#[derive(Deserialize, Debug)]
pub struct ServerMembersQuery {
    last_id: u64,
}

#[get("/servers/{server_id}/members")]
async fn server_members(
    data: web::Data<AppState>,
    path: web::Path<u64>,
    query: Option<web::Query<ServerMembersQuery>>,
    session: web::ReqData<Session>,
) -> Result<impl Responder, Error> {
    let server_id = path.into_inner();

    let member = query!("SELECT id FROM Member WHERE user_id = ? AND server_id = ?", session.user_id, server_id)
        .fetch_optional(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    if member.is_none() {
        return Ok(HttpResponse::NotFound().finish());
    }

    let members = query!(
        "SELECT Member.id, Member.created_at, Member.server_id, Member.user_id, Member.nickname, User.created_at AS user_created_at, User.username AS user_username FROM Member INNER JOIN User ON User.id = Member.user_id WHERE Member.server_id = ? AND Member.id > ? ORDER BY Member.id LIMIT 100",
        server_id,
        query.map(|q| q.into_inner().last_id).unwrap_or(0)
    )
        .fetch_all(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    Ok(HttpResponse::Ok().json(members.iter().map(|record| {
        let member = structures::member::Member {
            id: record.id.into(),
            created_at: record.created_at,
            server_id: record.server_id.into(),
            user_id: record.user_id.into(),
            nickname: record.nickname.clone(),
        };

        let mut member_value = serde_json::to_value(member.clone()).unwrap();

        if let Some(id) = member.user_id.0 {
            let user = structures::user::User {
                id: id.into(),
                created_at: record.user_created_at,
                username: record.user_username.clone(),
                password: "".to_string(),
            };

            merge_json(
                &mut member_value,
                &serde_json::json!({ "user": user }),
            );
        };

        member_value
    }).collect::<serde_json::Value>()))
}

#[get("/servers/{server_id}/members/{member_id}")]
async fn server_member(
    data: web::Data<AppState>,
    path: web::Path<(u64, u64)>,
) -> Result<impl Responder, Error> {
    let (server_id, member_id) = path.into_inner();

    let record_opt = query!(
        "SELECT Member.id, Member.created_at, Member.server_id, Member.user_id, Member.nickname, User.created_at AS user_created_at, User.username AS user_username FROM Member INNER JOIN User ON User.id = Member.user_id WHERE Member.server_id = ? AND Member.id = ? LIMIT 100",
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

    let member = structures::member::Member {
        id: record.id.into(),
        created_at: record.created_at,
        server_id: record.server_id.into(),
        user_id: record.user_id.into(),
        nickname: record.nickname.clone(),
    };

    let mut member_value = serde_json::to_value(member.clone()).unwrap();

    if let Some(id) = member.user_id.0 {
        let user = structures::user::User {
            id: id.into(),
            created_at: record.user_created_at,
            username: record.user_username.clone(),
            password: "".to_string(),
        };

        merge_json(
            &mut member_value,
            &serde_json::json!({ "user": user }),
        );
    };

    Ok(HttpResponse::Ok().json(member_value))
}

#[post("/servers/{server_id}/leave")]
async fn leave_server(
    data: web::Data<AppState>,
    path: web::Path<u64>,
    session: web::ReqData<Session>,
) -> Result<impl Responder, Error> {
    let server_id = path.into_inner();

    let record_opt = query!(
        "SELECT Member.id, Member.created_at, Member.server_id, Member.user_id, Member.nickname, User.created_at AS user_created_at, User.username AS user_username FROM Member INNER JOIN User ON User.id = Member.user_id WHERE Member.server_id = ? AND Member.user_id = ?",
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
        record.id,
        server_id
    )
        .execute(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    let member = structures::member::Member {
        id: record.id.into(),
        created_at: record.created_at,
        server_id: record.server_id.into(),
        user_id: record.user_id.into(),
        nickname: record.nickname.clone(),
    };

    let mut member_value = serde_json::to_value(member.clone()).unwrap();

    if let Some(id) = member.user_id.0 {
        let user = structures::user::User {
            id: id.into(),
            created_at: record.user_created_at,
            username: record.user_username.clone(),
            password: "".to_string(),
        };

        merge_json(
            &mut member_value,
            &serde_json::json!({ "user": user }),
        );
    };

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
    }

    data.server_connections.entry(server_id).and_modify(|map| {
        map.remove(&session.user_id.0);
    });

    Ok(HttpResponse::Ok().json(member_value))
}