use actix_web::{error::Error, get, post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Map;
use sqlx::{query, query_as};
use std::str::FromStr;
use validator::Validate;

use crate::consts::{merge_json, send_to_server_members};
use crate::structures::message::MessageKind;
use crate::ws::JsonMessage;
use crate::{
    errors,
    structures::{self, session::Session},
    AppState,
};

#[derive(Deserialize, Debug)]
pub struct ChannelMessagesQuery {
    last_id: Option<u64>,
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

    let messages = query!(
        "SELECT Message.id, Message.created_at, Message.content, Message.kind, Message.channel_id, Message.member_id, Member.created_at AS member_created_at, Member.server_id AS member_server_id, Member.user_id AS member_user_id, Member.nickname AS member_nickname, User.created_at AS user_created_at, User.username AS user_username FROM Message INNER JOIN Member ON Member.id = Message.member_id INNER JOIN User ON User.id = Member.user_id WHERE Message.channel_id = ? AND Message.id > ? ORDER BY Message.id LIMIT 100",
        channel_id,
        query.map(|q| q.into_inner().last_id.unwrap_or(0)).unwrap_or(0)
    )
        .fetch_all(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    Ok(HttpResponse::Ok().json(
        messages
            .iter()
            .map(|record| {
                let message = structures::message::Message {
                    id: record.id.into(),
                    created_at: record.created_at,
                    content: record.content.clone(),
                    kind: MessageKind::from_str(record.kind.as_str()).unwrap(),
                    channel_id: record.channel_id.into(),
                    member_id: record.member_id.into(),
                };

                let mut message_value = serde_json::to_value(message.clone()).unwrap();

                let member = structures::member::Member {
                    id: record.member_id.into(),
                    created_at: record.member_created_at,
                    server_id: record.member_server_id.into(),
                    user_id: record.member_user_id.into(),
                    nickname: record.member_nickname.clone(),
                };

                let mut member_value = serde_json::to_value(member.clone()).unwrap();

                if let Some(id) = member.user_id.0 {
                    let user = structures::user::User {
                        id: id.into(),
                        created_at: record.user_created_at,
                        username: record.user_username.clone(),
                        password: "".to_string(),
                    };

                    merge_json(&mut member_value, &serde_json::json!({ "user": user }));
                }

                let mut map = Map::new();
                map.insert("member".to_string(), member_value);

                merge_json(&mut message_value, &serde_json::Value::Object(map));

                message_value
            })
            .collect::<serde_json::Value>(),
    ))
}

#[derive(Deserialize, Validate)]
pub struct CreateChannelData {
    #[validate(length(min = 1, max = 4500))]
    content: String,
}

#[post("/servers/{server_id}/channels/{channel_id}/messages")]
async fn create_message(
    data: web::Data<AppState>,
    path: web::Path<(u64, u64)>,
    req_body: web::Either<web::Json<CreateChannelData>, web::Form<CreateChannelData>>,
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
        "SELECT Message.id, Message.created_at, Message.content, Message.kind, Message.channel_id, Message.member_id, Member.created_at AS member_created_at, Member.server_id AS member_server_id, Member.user_id AS member_user_id, Member.nickname AS member_nickname, User.created_at AS user_created_at, User.username AS user_username FROM Message INNER JOIN Member ON Member.id = Message.member_id INNER JOIN User ON User.id = Member.user_id WHERE Message.channel_id = ? AND Message.id = ? LIMIT 100",
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

    let mut message_value = serde_json::to_value(message_struct.clone()).unwrap();

    let member = structures::member::Member {
        id: record.member_id.into(),
        created_at: record.member_created_at,
        server_id: record.member_server_id.into(),
        user_id: record.member_user_id.into(),
        nickname: record.member_nickname.clone(),
    };

    let mut member_value = serde_json::to_value(member.clone()).unwrap();

    if let Some(id) = member.user_id.0 {
        let user = structures::user::User {
            id: id.into(),
            created_at: record.user_created_at,
            username: record.user_username.clone(),
            password: "".to_string(),
        };

        merge_json(&mut member_value, &serde_json::json!({ "user": user }));
    }

    let mut map = Map::new();
    map.insert("member".to_string(), member_value);

    merge_json(&mut message_value, &serde_json::Value::Object(map));

    Ok(HttpResponse::Ok().json(message_value))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(channel_messages)
        .service(create_message)
        .service(channel_message);
}
