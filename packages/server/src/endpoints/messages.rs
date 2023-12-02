use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{query_as, query_as_unchecked};
use std::fmt::Debug;
use validator::Validate;

use crate::consts::{get_channel_access_data, merge_json, send_to_server_members, send_to_users};
use crate::errors::RouteError;
use crate::id_type::OptionId;
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

struct MessageResult {
    id: u64,
    created_at: DateTime<Utc>,
    content: String,
    kind: MessageKind,
    channel_id: u64,
    user_id: Option<u64>,
    user_created_at: Option<DateTime<Utc>>,
    user_username: Option<String>,
    member_id: Option<u64>,
    member_created_at: Option<DateTime<Utc>>,
    member_server_id: Option<u64>,
    member_nickname: Option<String>,
}

fn messageresult_to_message(record: &MessageResult) -> serde_json::Value {
    let mut message = serde_json::to_value(structures::message::Message {
        id: record.id.into(),
        created_at: record.created_at,
        content: record.content.clone(),
        kind: record.kind.clone(),
        channel_id: record.channel_id.into(),
        user_id: OptionId(record.user_id),
    })
    .unwrap();

    if let (Some(id), Some(created_at), Some(username)) = (
        record.user_id,
        record.user_created_at,
        record.user_username.clone(),
    ) {
        merge_json(
            &mut message,
            &serde_json::json!({
                "user": structures::user::User {
                    id: id.into(),
                    created_at,
                    username,
                    password: "".to_string()
                }
            }),
        );
    }

    if let (Some(id), Some(created_at), Some(server_id)) = (
        record.member_id,
        record.member_created_at,
        record.member_server_id,
    ) {
        merge_json(
            &mut message,
            &serde_json::json!({
                "member": structures::member::Member {
                    id: id.into(),
                    created_at,
                    server_id: server_id.into(),
                    user_id: OptionId(record.user_id),
                    nickname: record.member_nickname.clone()
                }
            }),
        );
    }

    message
}

#[get("/channels/{channel_id}/messages")]
async fn channel_messages(
    data: web::Data<AppState>,
    path: web::Path<u64>,
    query: Option<web::Query<ChannelMessagesQuery>>,
    session: web::ReqData<Session>,
) -> Result<impl Responder, RouteError> {
    let channel_id = path.into_inner();

    get_channel_access_data(&data, channel_id, session.user_id.0).await?;

    let last_id_opt = query.map(|q| q.into_inner().last_id).unwrap_or(None);

    let mut messages = match last_id_opt {
        Some(last_id) => query_as_unchecked!(
            MessageResult,
            "SELECT Message.id, Message.created_at, Message.content, Message.kind AS `kind: _`, Message.channel_id, Message.user_id, User.created_at AS user_created_at, User.username AS user_username, Member.id as member_id, Member.created_at AS member_created_at, Member.server_id AS member_server_id, Member.nickname AS member_nickname FROM Message LEFT JOIN User ON User.id = Message.user_id LEFT JOIN Member ON Member.server_id = (SELECT server_id FROM Channel WHERE id = Message.channel_id) AND Member.user_id = Message.user_id WHERE Message.channel_id = ? AND Message.id < ? ORDER BY Message.id DESC LIMIT 100",
            channel_id,
            last_id
        )
            .fetch_all(&data.db)
            .await
            .map_err(|e| RouteError::Errors(errors::Errors::Db(e)))?,
        None => query_as_unchecked!(
            MessageResult,
            "SELECT Message.id, Message.created_at, Message.content, Message.kind AS `kind: _`, Message.channel_id, Message.user_id, User.created_at AS user_created_at, User.username AS user_username, Member.id as member_id, Member.created_at AS member_created_at, Member.server_id AS member_server_id, Member.nickname AS member_nickname FROM Message LEFT JOIN User ON User.id = Message.user_id LEFT JOIN Member ON Member.server_id = (SELECT server_id FROM Channel WHERE id = Message.channel_id) AND Member.user_id = Message.user_id WHERE Message.channel_id = ? ORDER BY Message.id DESC LIMIT 100",
            channel_id
        )
            .fetch_all(&data.db)
            .await
            .map_err(|e| RouteError::Errors(errors::Errors::Db(e)))?
    };

    messages.reverse();

    Ok(HttpResponse::Ok().json(
        messages
            .iter()
            .map(|record| messageresult_to_message(record))
            .collect::<serde_json::Value>(),
    ))
}

#[derive(Deserialize, Validate)]
pub struct CreateMessageData {
    #[validate(length(min = 1, max = 2000))]
    content: String,
}

#[post("/channels/{channel_id}/messages")]
async fn create_message(
    data: web::Data<AppState>,
    path: web::Path<u64>,
    req_body: web::Either<web::Json<CreateMessageData>, web::Form<CreateMessageData>>,
    session: web::ReqData<Session>,
) -> Result<impl Responder, RouteError> {
    let channel_id = path.into_inner();

    let (channel, channel_recipients, member_opt, user) =
        get_channel_access_data(&data, channel_id, session.user_id.0).await?;

    let content = req_body.into_inner().content;

    let message: (u64, DateTime<Utc>) =
        query_as("INSERT INTO Message VALUES (NULL, DEFAULT, ?, ?, ?, ?) RETURNING id, created_at")
            .bind(content.clone())
            .bind(MessageKind::Text)
            .bind(channel_id)
            .bind(session.user_id)
            .fetch_one(&data.db)
            .await
            .map_err(|e| RouteError::Errors(errors::Errors::Db(e)))?;

    let message_struct = structures::message::Message {
        id: message.0.into(),
        created_at: message.1,
        content,
        kind: MessageKind::Text,
        channel_id: channel_id.into(),
        user_id: OptionId(Some(session.user_id.0)),
    };

    let mut message_value = serde_json::to_value(message_struct.clone()).unwrap();

    if let Some(member) = member_opt {
        merge_json(&mut message_value, &serde_json::json!({ "member": member }));
    }

    merge_json(&mut message_value, &serde_json::json!({ "user": user }));

    let json_message = JsonMessage(serde_json::json!({
        "type": "message_create",
        "data": message_value.clone()
    }));

    if let Some(server_id) = channel.server_id.0 {
        send_to_server_members(&data, server_id, &json_message);
    }

    if let Some(channel_recipients) = channel_recipients {
        send_to_users(&data, channel_recipients.into_iter(), &json_message);
    }

    Ok(HttpResponse::Ok().json(message_value))
}

#[get("/channels/{channel_id}/messages/{message_id}")]
async fn channel_message(
    data: web::Data<AppState>,
    path: web::Path<(u64, u64)>,
    session: web::ReqData<Session>,
) -> Result<impl Responder, RouteError> {
    let (channel_id, message_id) = path.into_inner();

    get_channel_access_data(&data, channel_id, session.user_id.0).await?;

    match query_as_unchecked!(
        MessageResult,
        "SELECT Message.id, Message.created_at, Message.content, Message.kind AS `kind: _`, Message.channel_id, Message.user_id, User.created_at AS user_created_at, User.username AS user_username, Member.id as member_id, Member.created_at AS member_created_at, Member.server_id AS member_server_id, Member.nickname AS member_nickname FROM Message LEFT JOIN User ON User.id = Message.user_id LEFT JOIN Member ON Member.server_id = (SELECT server_id FROM Channel WHERE id = Message.channel_id) AND Member.user_id = Message.user_id WHERE Message.id = ? AND Message.channel_id = ?",
        message_id,
        channel_id,
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|e| RouteError::Errors(errors::Errors::Db(e)))? {
        Some(record) => Ok(HttpResponse::Ok().json(messageresult_to_message(&record))),
        None => Ok(HttpResponse::NotFound().finish())
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(channel_messages)
        .service(create_message)
        .service(channel_message);
}
