use crate::{
    errors::{self, RouteError},
    structures::{self, channel::Channel, member::Member, user::User},
    ws::JsonMessage,
    AppState,
};
use actix_web::http::StatusCode;
use actix_web::web;
use serde_json::Value;
use sqlx::{query, query_as, query_as_unchecked};

pub const SESSION_COOKIE_NAME: &str = "biasdo_SESSION";

pub fn merge_json(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                merge_json(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}

pub fn send_to_users<T, I>(data: &web::Data<AppState>, user_ids: I, message: &JsonMessage<T>)
where
    T: serde::Serialize + std::fmt::Debug + Clone + Send + 'static,
    I: Iterator<Item = u64>,
{
    for user_id in user_ids {
        if let Some(user_sockets) = data.user_connections.get(&user_id) {
            user_sockets.iter().for_each(|addr| {
                addr.do_send(message.clone());
            });
        }
    }
}

pub fn send_to_server_members<T>(
    data: &web::Data<AppState>,
    server_id: u64,
    message: &JsonMessage<T>,
) where
    T: serde::Serialize + std::fmt::Debug + Clone + Send + 'static,
{
    if let Some(server_connections) = data.server_connections.get(&server_id) {
        send_to_users(
            data,
            server_connections.value().clone().into_iter(),
            message,
        )
    }
}

#[macro_export]
macro_rules! add_value_to_json {
    ($into:expr, $thing:expr, $key:expr) => {{
        let mut into_value = serde_json::to_value($into.clone()).unwrap();
        let thing_value = serde_json::to_value($thing.clone()).unwrap();

        let mut map = serde_json::Map::new();
        map.insert($key.to_string(), thing_value);

        crate::consts::merge_json(&mut into_value, &serde_json::Value::Object(map));

        into_value
    }};
}

pub async fn get_channel_access_data(
    data: &web::Data<AppState>,
    channel_id: u64,
    user_id: u64,
) -> Result<(Channel, Option<Vec<u64>>, Option<Member>, User), RouteError> {
    let channel = match query_as_unchecked!(
        structures::channel::Channel,
        "SELECT id, created_at, name, kind as `kind: _`, server_id FROM Channel WHERE id = ?",
        channel_id
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|e| RouteError::Errors(errors::Errors::Db(e)))?
    {
        Some(channel) => channel,
        None => return Err(RouteError::Status(StatusCode::NOT_FOUND)),
    };

    let channel_recipients = match channel.kind.has_recipients() {
        true => Some(
            query!(
                "SELECT user_id FROM ChannelRecipient WHERE channel_id = ?",
                channel_id
            )
            .fetch_all(&data.db)
            .await
            .map_err(|e| RouteError::Errors(errors::Errors::Db(e)))?
            .into_iter()
            .map(|r| r.user_id)
            .collect::<Vec<u64>>(),
        ),
        false => None,
    };

    let member_opt = match channel.kind.has_members() {
        true => match query_as!(
            structures::member::Member,
            "SELECT id, created_at, server_id, nickname, user_id FROM Member WHERE user_id = ? AND server_id = ?",
            user_id,
            channel.server_id
        )
            .fetch_optional(&data.db)
            .await
            .map_err(|e| RouteError::Errors(errors::Errors::Db(e)))?
        {
            Some(member) => Some(member),
            None => return Err(RouteError::Status(StatusCode::NOT_FOUND)),
        },
        false => None,
    };

    let user = query_as!(
        structures::user::User,
        "SELECT id, created_at, username, password FROM User WHERE id = ?",
        user_id
    )
    .fetch_one(&data.db)
    .await
    .map_err(|e| RouteError::Errors(errors::Errors::Db(e)))?;

    return Ok((channel, channel_recipients, member_opt, user));
}
