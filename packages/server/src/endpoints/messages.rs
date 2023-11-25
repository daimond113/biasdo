use actix_web::{error::Error, get, post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Map;
use sqlx::{query, query_as, query_as_unchecked};
use std::str::FromStr;
use validator::Validate;

use crate::consts::{merge_json, send_to_server_members};
use crate::structures::message::MessageKind;
use crate::ws::JsonMessage;
use crate::{
    add_value_to_json, errors, get_member_opt_user_value,
    structures::{self, session::Session},
    AppState,
};

#[derive(Deserialize, Debug)]
pub struct ChannelMessagesQuery {
    last_id: Option<u64>,
}

struct MessageResult {
    id: u64,
    created_at: DateTime<Utc>,
    content: String,
    kind: MessageKind,
    channel_id: u64,
    member_id: u64,
    member_created_at: DateTime<Utc>,
    member_server_id: u64,
    member_user_id: Option<u64>,
    member_nickname: Option<String>,
    user_created_at: Option<DateTime<Utc>>,
    user_username: Option<String>,
}

#[get("/servers/{server_id}/channels/{channel_id}/messages")]
async fn channel_messages(
    data: web::Data<AppState>,
    path: web::Path<(u64, u64)>,
    query: Option<web::Query<ChannelMessagesQuery>>,
    session: web::ReqData<Session>,
) -> Result<impl Responder, Error> {
    let (server_id, channel_id) = path.into_inner();

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

    let channel = query!(
        "SELECT id FROM Channel WHERE id = ? AND server_id = ?",
        channel_id,
        server_id
    )
    .fetch_optional(&data.db)
    .await
    .map_err(errors::Errors::Db)?;

    if channel.is_none() {
        return Ok(HttpResponse::NotFound().finish());
    }

    let last_id_opt = query.map(|q| q.into_inner().last_id).unwrap_or(None);

    let mut messages = match last_id_opt {
        Some(last_id) => query_as_unchecked!(
            MessageResult,
            "SELECT Message.id, Message.created_at, Message.content, Message.kind AS `kind: _`, Message.channel_id, Message.member_id, Member.created_at AS member_created_at, Member.server_id AS member_server_id, Member.user_id AS member_user_id, Member.nickname AS member_nickname, User.created_at AS user_created_at, User.username AS user_username FROM Message INNER JOIN Member ON Member.id = Message.member_id LEFT JOIN User ON User.id = Member.user_id WHERE Message.channel_id = ? AND Message.id < ? ORDER BY Message.id DESC LIMIT 100",
            channel_id,
            last_id
        )
            .fetch_all(&data.db)
            .await
            .map_err(errors::Errors::Db)?,
        None => query_as_unchecked!(
            MessageResult,
            "SELECT Message.id, Message.created_at, Message.content, Message.kind AS `kind: _`, Message.channel_id, Message.member_id, Member.created_at AS member_created_at, Member.server_id AS member_server_id, Member.user_id AS member_user_id, Member.nickname AS member_nickname, User.created_at AS user_created_at, User.username AS user_username FROM Message INNER JOIN Member ON Member.id = Message.member_id LEFT JOIN User ON User.id = Member.user_id WHERE Message.channel_id = ? ORDER BY Message.id DESC LIMIT 100",
            channel_id
        )
            .fetch_all(&data.db)
            .await
            .map_err(errors::Errors::Db)?
    };

    messages.reverse();

    Ok(HttpResponse::Ok().json(
        messages
            .iter()
            .map(|record| {
                add_value_to_json!(
                    structures::message::Message {
                        id: record.id.into(),
                        created_at: record.created_at,
                        content: record.content.clone(),
                        kind: record.kind.clone(),
                        channel_id: record.channel_id.into(),
                        member_id: record.member_id.into(),
                    },
                    get_member_opt_user_value!(record),
                    "member"
                )
            })
            .collect::<serde_json::Value>(),
    ))
}

#[derive(Deserialize, Validate)]
pub struct CreateMessageData {
    #[validate(length(min = 1, max = 2000))]
    content: String,
}

#[post("/servers/{server_id}/channels/{channel_id}/messages")]
async fn create_message(
    data: web::Data<AppState>,
    path: web::Path<(u64, u64)>,
    req_body: web::Either<web::Json<CreateMessageData>, web::Form<CreateMessageData>>,
    session: web::ReqData<Session>,
) -> Result<impl Responder, Error> {
    let (server_id, channel_id) = path.into_inner();

    let member_opt = query_as!(
        structures::member::Member,
        "SELECT id, created_at, server_id, nickname, user_id FROM Member WHERE user_id = ? AND server_id = ?",
        session.user_id,
        server_id
    )
        .fetch_optional(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    if member_opt.is_none() {
        return Ok(HttpResponse::NotFound().finish());
    }

    let channel = query!(
        "SELECT id FROM Channel WHERE id = ? AND server_id = ?",
        channel_id,
        server_id
    )
    .fetch_optional(&data.db)
    .await
    .map_err(errors::Errors::Db)?;

    if channel.is_none() {
        return Ok(HttpResponse::NotFound().finish());
    }

    let content = req_body.into_inner().content;

    let member = member_opt.unwrap();

    let message: (u64, DateTime<Utc>) =
        query_as("INSERT INTO Message VALUES (NULL, DEFAULT, ?, ?, ?, ?) RETURNING id, created_at")
            .bind(content.clone())
            .bind(MessageKind::Text)
            .bind(channel_id)
            .bind(member.id)
            .fetch_one(&data.db)
            .await
            .map_err(errors::Errors::Db)?;

    let message_struct = structures::message::Message {
        id: message.0.into(),
        created_at: message.1,
        content,
        kind: MessageKind::Text,
        channel_id: channel_id.into(),
        member_id: member.id.into(),
    };

    let mut message_value = serde_json::to_value(message_struct.clone()).unwrap();

    let mut member_value = serde_json::to_value(member.clone()).unwrap();

    if let Some(id) = member.user_id.0 {
        let user = query_as!(
            structures::user::User,
            "SELECT id, created_at, username, password FROM User WHERE id = ?",
            id
        )
        .fetch_one(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

        merge_json(&mut member_value, &serde_json::json!({ "user": user }));
    }

    let mut map = Map::new();
    map.insert("member".to_string(), member_value);

    merge_json(&mut message_value, &serde_json::Value::Object(map));

    send_to_server_members(
        &data,
        server_id,
        &JsonMessage(serde_json::json!({
            "type": "message_create",
            "data": message_value.clone()
        })),
    );

    Ok(HttpResponse::Ok().json(message_value))
}

#[get("/servers/{server_id}/channels/{channel_id}/messages/{message_id}")]
async fn channel_message(
    data: web::Data<AppState>,
    path: web::Path<(u64, u64, u64)>,
    session: web::ReqData<Session>,
) -> Result<impl Responder, Error> {
    let (server_id, channel_id, message_id) = path.into_inner();

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

    let channel = query!(
        "SELECT id FROM Channel WHERE id = ? AND server_id = ?",
        channel_id,
        server_id
    )
    .fetch_optional(&data.db)
    .await
    .map_err(errors::Errors::Db)?;

    if channel.is_none() {
        return Ok(HttpResponse::NotFound().finish());
    }

    let message = query!(
        "SELECT Message.id, Message.created_at, Message.content, Message.kind, Message.channel_id, Message.member_id, Member.created_at AS member_created_at, Member.server_id AS member_server_id, Member.user_id AS member_user_id, Member.nickname AS member_nickname, User.created_at AS user_created_at, User.username AS user_username FROM Message INNER JOIN Member ON Member.id = Message.member_id LEFT JOIN User ON User.id = Member.user_id WHERE Message.channel_id = ? AND Message.id = ?",
        channel_id,
        message_id
    )
        .fetch_optional(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    if message.is_none() {
        return Ok(HttpResponse::NotFound().finish());
    }

    let record = message.unwrap();

    let message_struct = structures::message::Message {
        id: record.id.into(),
        created_at: record.created_at,
        content: record.content.clone(),
        kind: MessageKind::from_str(record.kind.as_str()).unwrap(),
        channel_id: record.channel_id.into(),
        member_id: record.member_id.into(),
    };

    Ok(HttpResponse::Ok().json(add_value_to_json!(
        message_struct,
        get_member_opt_user_value!(record),
        "member"
    )))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(channel_messages)
        .service(create_message)
        .service(channel_message);
}
