use std::{collections::HashSet, sync::Mutex};

use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{query, query_as};
use validator::Validate;

use crate::{
	error::ApiResult,
	middleware::Identity,
	models::{
		message::{Message, MessageKind},
		scope::{ReadWrite, Scope},
		servermember::ServerMember,
		user::User,
	},
	update_structure,
	ws::{send_updates, WsUpdateEvent},
	AppState,
};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateMessageBody {
	#[serde(deserialize_with = "super::trim_string")]
	#[validate(length(min = 1, max = 3500))]
	content: String,
}

pub async fn create_message(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	generator: web::Data<Mutex<snowflaked::Generator>>,
	body: web::Json<CreateMessageBody>,
	path: web::Path<u64>,
) -> ApiResult {
	body.validate()?;

	let Some(user_id) = identity.is_user_like_with_scope(Scope::Messages(ReadWrite::Write)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let channel_id = path.into_inner();

	let (recipients, member) = {
		let rows = query!(
            r#"SELECT ServerMember.server_id, ServerMember.nickname, ServerMember.created_at AS `created_at: DateTime<Utc>`,
DMChannelRecipient.user_id
FROM Channel
LEFT JOIN ServerMember ON ServerMember.server_id=Channel.server_id AND ServerMember.user_id=?
LEFT JOIN DMChannelRecipient ON DMChannelRecipient.channel_id=Channel.id
WHERE Channel.id = ?
"#,
            user_id,
            channel_id,
        )
        .fetch_all(&app_state.db)
        .await?;

		let Some(channel_row) = rows.first() else {
			return Ok(HttpResponse::Forbidden().finish());
		};

		let recipients = rows
			.iter()
			.filter_map(|row| row.user_id)
			.collect::<HashSet<_>>();

		if !recipients.contains(&user_id) && channel_row.server_id.is_none() {
			return Ok(HttpResponse::Forbidden().finish());
		}

		(
			channel_row
				.server_id
				.and_then(|server_id| app_state.server_connections.get(&server_id))
				.map(|conns| conns.clone())
				.or(Some(recipients)),
			channel_row.created_at.map(|created_at| ServerMember {
				user_id,
				server_id: channel_row.server_id.unwrap(),
				nickname: channel_row.nickname.clone(),
				created_at,
				user: None,
			}),
		)
	};

	let message_id: u64 = {
		let mut generator = generator.lock().unwrap();
		generator.generate()
	};

	let content = body.content.clone();

	let user = query_as!(
		User,
		"SELECT id, username, display_name FROM User WHERE id = ?",
		user_id
	)
	.fetch_one(&app_state.db)
	.await?;

	query!(
        "INSERT INTO ChannelMessage (id, updated_at, content, kind, channel_id, user_id) VALUES (?, NULL, ?, 'text', ?, ?)",
        message_id,
        content,
        channel_id,
        user_id
    )
    .execute(&app_state.db)
    .await?;

	let message = Message {
		id: message_id,
		updated_at: None,
		content: content.clone(),
		kind: MessageKind::Text,
		channel_id,
		user: user.clone(),
		member: member.clone(),
	};

	if let Some(recipients) = recipients {
		send_updates(
			[WsUpdateEvent::MessageCreate(message.clone())],
			&app_state,
			recipients,
		);
	}

	Ok(HttpResponse::Created().json(message))
}

#[derive(Debug, Deserialize, Validate)]
pub struct GetMessagesQuery {
	#[validate(range(min = 1, max = 100))]
	limit: Option<u64>,
	last_id: Option<u64>,
}

macro_rules! message_row {
	($server_id:expr, $row:expr) => {{
		let user = User {
			id: $row.user_id,
			username: $row.username,
			display_name: $row.display_name,
		};

		let member = $row.created_at.map(|created_at| ServerMember {
			user_id: $row.user_id,
			server_id: $server_id.unwrap(),
			nickname: $row.nickname,
			created_at,
			user: None,
		});

		Message {
			id: $row.id,
			updated_at: $row.updated_at,
			content: $row.content,
			kind: $row.kind.parse().unwrap(),
			channel_id: $row.channel_id,
			user,
			member,
		}
	}};
}

pub async fn get_messages(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	path: web::Path<u64>,
	query: web::Query<GetMessagesQuery>,
) -> ApiResult {
	query.validate()?;

	let Some(user_id) = identity.is_user_like_with_scope(Scope::Messages(ReadWrite::Read)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let channel_id = path.into_inner();

	let server_id = {
		let rows = query!(
			r#"SELECT ServerMember.server_id, DMChannelRecipient.user_id
FROM Channel
LEFT JOIN ServerMember ON ServerMember.server_id=Channel.server_id AND ServerMember.user_id=?
LEFT JOIN DMChannelRecipient ON DMChannelRecipient.channel_id=Channel.id
WHERE Channel.id = ?
"#,
			user_id,
			channel_id,
		)
		.fetch_all(&app_state.db)
		.await?;

		let Some(channel_row) = rows.first() else {
			return Ok(HttpResponse::Forbidden().finish());
		};

		let recipients = rows
			.iter()
			.filter_map(|row| row.user_id)
			.collect::<HashSet<_>>();

		if !recipients.contains(&user_id) && channel_row.server_id.is_none() {
			return Ok(HttpResponse::Forbidden().finish());
		}

		channel_row.server_id
	};

	let limit = query.limit.unwrap_or(50);
	let last_id = query.last_id.unwrap_or(u64::MAX);

	let mut messages = query!(
        r#"SELECT ChannelMessage.id, ChannelMessage.updated_at, ChannelMessage.content, ChannelMessage.kind, ChannelMessage.channel_id, ChannelMessage.user_id,
User.username, User.display_name,
ServerMember.nickname, ServerMember.created_at
FROM ChannelMessage
INNER JOIN User ON User.id=ChannelMessage.user_id
LEFT JOIN ServerMember ON ServerMember.user_id=ChannelMessage.user_id AND ServerMember.server_id=?
WHERE ChannelMessage.channel_id = ? AND ChannelMessage.id < ?
ORDER BY id DESC
LIMIT ?
"#,
        server_id,
        channel_id,
        last_id,
        limit
    )
    .fetch_all(&app_state.db)
    .await?;

	messages.reverse();

	Ok(HttpResponse::Ok().json(
		messages
			.into_iter()
			.map(|row| message_row!(server_id, row))
			.collect::<Vec<_>>(),
	))
}

pub async fn get_message(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	path: web::Path<(u64, u64)>,
) -> ApiResult {
	let Some(user_id) = identity.is_user_like_with_scope(Scope::Messages(ReadWrite::Read)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let (channel_id, message_id) = path.into_inner();

	let server_id = {
		let rows = query!(
			r#"SELECT ServerMember.server_id, DMChannelRecipient.user_id
FROM Channel
LEFT JOIN ServerMember ON ServerMember.server_id=Channel.server_id AND ServerMember.user_id=?
LEFT JOIN DMChannelRecipient ON DMChannelRecipient.channel_id=Channel.id
WHERE Channel.id = ?
"#,
			user_id,
			channel_id,
		)
		.fetch_all(&app_state.db)
		.await?;

		let Some(channel_row) = rows.first() else {
			return Ok(HttpResponse::Forbidden().finish());
		};

		let recipients = rows
			.iter()
			.filter_map(|row| row.user_id)
			.collect::<HashSet<_>>();

		if !recipients.contains(&user_id) && channel_row.server_id.is_none() {
			return Ok(HttpResponse::Forbidden().finish());
		}

		channel_row.server_id
	};

	let Some(message) = query!(
        r#"SELECT ChannelMessage.id, ChannelMessage.updated_at, ChannelMessage.content, ChannelMessage.kind, ChannelMessage.channel_id, ChannelMessage.user_id,
User.username, User.display_name,
ServerMember.nickname, ServerMember.created_at
FROM ChannelMessage
INNER JOIN User ON User.id=ChannelMessage.user_id
LEFT JOIN ServerMember ON ServerMember.user_id=ChannelMessage.user_id AND ServerMember.server_id=?
WHERE ChannelMessage.id = ? AND ChannelMessage.channel_id = ?
"#,
        server_id,
        message_id,
        channel_id
    )
    .fetch_optional(&app_state.db)
    .await? else {
        return Ok(HttpResponse::NotFound().finish());
    };

	Ok(HttpResponse::Ok().json(message_row!(server_id, message)))
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMessageBody {
	#[serde(default, deserialize_with = "super::trim_opt_string")]
	#[validate(length(min = 1, max = 3500))]
	content: Option<String>,
}

pub async fn update_message(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	path: web::Path<(u64, u64)>,
	body: web::Json<UpdateMessageBody>,
) -> ApiResult {
	body.validate()?;

	let Some(user_id) = identity.is_user_like_with_scope(Scope::Messages(ReadWrite::Write)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let (channel_id, message_id) = path.into_inner();

	let recipients = {
		let rows = query!(
			r#"SELECT ServerMember.server_id, DMChannelRecipient.user_id
FROM Channel
LEFT JOIN ServerMember ON ServerMember.server_id=Channel.server_id AND ServerMember.user_id=?
LEFT JOIN DMChannelRecipient ON DMChannelRecipient.channel_id=Channel.id
WHERE Channel.id = ?
"#,
			user_id,
			channel_id,
		)
		.fetch_all(&app_state.db)
		.await?;

		let Some(channel_row) = rows.first() else {
			return Ok(HttpResponse::Forbidden().finish());
		};

		let recipients = rows
			.iter()
			.filter_map(|row| row.user_id)
			.collect::<HashSet<_>>();

		if !recipients.contains(&user_id) && channel_row.server_id.is_none() {
			return Ok(HttpResponse::Forbidden().finish());
		}

		channel_row
			.server_id
			.and_then(|server_id| app_state.server_connections.get(&server_id))
			.map(|conns| conns.clone())
			.or(Some(recipients))
	};

	let updated_at = Utc::now();

	let query = update_structure!("ChannelMessage", body, content)
		.push(", updated_at = ")
		.push_bind(updated_at.naive_utc())
		.push(" WHERE id = ")
		.push_bind(message_id)
		.push(" AND user_id = ")
		.push_bind(user_id)
		.push(" AND channel_id = ")
		.push_bind(channel_id)
		.build()
		.execute(&app_state.db)
		.await?;

	if query.rows_affected() == 0 {
		return Ok(HttpResponse::NotFound().finish());
	}

	if let Some(recipients) = recipients {
		send_updates(
			[WsUpdateEvent::MessageUpdate {
				id: message_id,
				content: body.content.clone(),
				updated_at,
			}],
			&app_state,
			recipients,
		);
	}

	Ok(HttpResponse::Ok().finish())
}

pub async fn delete_message(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	path: web::Path<(u64, u64)>,
) -> ApiResult {
	let Some(user_id) = identity.is_user_like_with_scope(Scope::Messages(ReadWrite::Write)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let (channel_id, message_id) = path.into_inner();

	let recipients = {
		let rows = query!(
			r#"SELECT ServerMember.server_id, DMChannelRecipient.user_id
FROM Channel
LEFT JOIN ServerMember ON ServerMember.server_id=Channel.server_id AND ServerMember.user_id=?
LEFT JOIN DMChannelRecipient ON DMChannelRecipient.channel_id=Channel.id
WHERE Channel.id = ?
"#,
			user_id,
			channel_id,
		)
		.fetch_all(&app_state.db)
		.await?;

		let Some(channel_row) = rows.first() else {
			return Ok(HttpResponse::Forbidden().finish());
		};

		let recipients = rows
			.iter()
			.filter_map(|row| row.user_id)
			.collect::<HashSet<_>>();

		if !recipients.contains(&user_id) && channel_row.server_id.is_none() {
			return Ok(HttpResponse::Forbidden().finish());
		}

		channel_row
			.server_id
			.and_then(|server_id| app_state.server_connections.get(&server_id))
			.map(|conns| conns.clone())
			.or(Some(recipients))
	};

	let result = query!(
		"DELETE FROM ChannelMessage WHERE id = ? AND user_id = ? AND channel_id = ?",
		message_id,
		user_id,
		channel_id
	)
	.execute(&app_state.db)
	.await?;

	if result.rows_affected() == 0 {
		return Ok(HttpResponse::NotFound().finish());
	}

	if let Some(recipients) = recipients {
		send_updates(
			[WsUpdateEvent::MessageDelete { id: message_id }],
			&app_state,
			recipients,
		);
	}

	Ok(HttpResponse::Ok().finish())
}
