use crate::consts::{merge_json, send_to_users};
use crate::id_type::OptionId;
use crate::structures::user::User;
use crate::ws::JsonMessage;
use crate::{
    errors,
    structures::{
        channel::{Channel, ChannelKind},
        session::Session,
    },
    AppState,
};
use actix_web::error::Error;
use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde_json::Value;
use sqlx::{query, query_as, query_as_unchecked};
use std::str::FromStr;

#[get("/direct-messages/channels")]
async fn get_dm_channels(
    data: web::Data<AppState>,
    session: web::ReqData<Session>,
) -> Result<impl Responder, Error> {
    let return_data = query!(
        "SELECT Channel.id, Channel.created_at, Channel.name, Channel.kind, Channel.server_id, User.id AS user_id, User.created_at AS user_created_at, User.username AS user_username FROM Channel INNER JOIN ChannelRecipient ON Channel.id = ChannelRecipient.channel_id AND ? IN (SELECT user_id FROM ChannelRecipient WHERE ChannelRecipient.channel_id = Channel.id) INNER JOIN User ON ChannelRecipient.user_id = User.id ORDER BY id",
        session.user_id.0
    )
    .fetch_all(&data.db)
    .await
    .map_err(errors::Errors::Db)?;

    let mut channel_recipient_map: IndexMap<Channel, Vec<User>> = IndexMap::new();

    return_data.iter().for_each(|row| {
        let channel = Channel {
            id: row.id.into(),
            created_at: row.created_at,
            name: row.name.clone(),
            kind: ChannelKind::from_str(row.kind.as_str()).unwrap(),
            server_id: OptionId(row.server_id),
        };

        channel_recipient_map
            .entry(channel)
            .or_insert_with(Vec::new)
            .push(User {
                id: row.user_id.into(),
                created_at: row.user_created_at,
                username: row.user_username.clone(),
                password: "".to_string(),
            });
    });

    Ok(HttpResponse::Ok().json(
        channel_recipient_map
            .iter()
            .map(|(channel, recipients)| {
                let mut channel_value = serde_json::to_value(channel.clone()).unwrap();
                merge_json(
                    &mut channel_value,
                    &serde_json::json!({ "recipients": recipients }),
                );
                channel_value
            })
            .collect::<Vec<Value>>(),
    ))
}

#[get("/direct-messages/channels/{channel_id}")]
async fn get_dm_channel(
    data: web::Data<AppState>,
    path: web::Path<u64>,
    session: web::ReqData<Session>,
) -> Result<impl Responder, Error> {
    let channel_id = path.into_inner();

    let channel = match query_as_unchecked!(
        Channel,
        "SELECT id, created_at, name, kind AS `kind: _`, server_id FROM Channel WHERE server_id IS NULL AND id IN (SELECT channel_id FROM ChannelRecipient WHERE user_id = ?) AND id = ?",
        session.user_id.0,
        channel_id
    )
    .fetch_optional(&data.db)
    .await
    .map_err(errors::Errors::Db)?
    {
        Some(channel) => channel,
        None => return Ok(HttpResponse::NotFound().finish()),
    };

    let recipients = query_as!(
        User,
        "SELECT id, created_at, username, password FROM User INNER JOIN ChannelRecipient ON ChannelRecipient.user_id = User.id WHERE channel_id = ?",
        channel_id
    )
    .fetch_all(&data.db)
    .await
    .map_err(errors::Errors::Db)?;

    let mut channel_value = serde_json::to_value(channel).unwrap();
    merge_json(
        &mut channel_value,
        &serde_json::json!({ "recipients": recipients }),
    );

    Ok(HttpResponse::Ok().json(channel_value))
}

#[post("/direct-messages/channel/{user_id}")]
async fn create_dm_channel(
    data: web::Data<AppState>,
    path: web::Path<u64>,
    session: web::ReqData<Session>,
) -> Result<impl Responder, Error> {
    let user_id = path.into_inner();

    match query_as_unchecked!(Channel, "SELECT id, created_at, name, kind AS `kind: _`, server_id FROM Channel WHERE server_id IS NULL AND id IN (SELECT channel_id FROM ChannelRecipient WHERE user_id = ?) AND id IN (SELECT channel_id FROM ChannelRecipient WHERE user_id = ?)", session.user_id.0, user_id)
        .fetch_optional(&data.db)
        .await
        .map_err(errors::Errors::Db)?
    {
        Some(channel) => Ok(HttpResponse::Ok().json(channel)),
        None => {
            let record: (u64, DateTime<Utc>) =
                query_as("INSERT INTO Channel VALUES (NULL, DEFAULT, ?, ?, NULL) RETURNING id, created_at")
                .bind("")
                .bind(ChannelKind::DM)
                .fetch_one(&data.db)
                .await
                .map_err(errors::Errors::Db)?;

            let mut channel = serde_json::to_value(Channel {
                id: record.0.into(),
                created_at: record.1,
                name: String::new(),
                kind: ChannelKind::DM,
                server_id: None.into(),
            }).unwrap();

            query!(
                "INSERT INTO ChannelRecipient VALUES (?, ?), (?, ?)",
                record.0,
                session.user_id.0,
                record.0,
                user_id
            )
            .execute(&data.db)
            .await
            .map_err(errors::Errors::Db)?;

            let recipients = query_as!(
                User,
                "SELECT id, created_at, username, password FROM User WHERE id = ? OR id = ?",
                session.user_id.0,
                user_id
            )
                .fetch_all(&data.db)
                .await
                .map_err(errors::Errors::Db)?;

            merge_json(&mut channel, &serde_json::json!({ "recipients": recipients }));

            send_to_users(
                &data,
                vec![session.user_id.0, user_id].into_iter(),
                &JsonMessage(serde_json::json!({
                    "type": "channel_create",
                    "data": channel.clone()
                })),
            );

            Ok(HttpResponse::Ok().json(channel))
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_dm_channels)
        .service(get_dm_channels)
        .service(create_dm_channel);
}
