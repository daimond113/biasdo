use crate::{
	error::{BackendError, ErrorResponse},
	middleware::Identity,
	models::{
		channel::{Channel, ChannelKind},
		scope::{HasScope, ReadWrite, Scope},
	},
	update_structure,
	ws::{send_updates, WsUpdateEvent},
	AppState,
};
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::query;
use std::sync::Mutex;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateChannelBody {
	#[serde(deserialize_with = "super::trim_string")]
	#[validate(length(min = 2, max = 32))]
	name: String,
}

pub async fn create_channel(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	generator: web::Data<Mutex<snowflaked::Generator>>,
	body: web::Json<CreateChannelBody>,
	path: web::Path<u64>,
) -> Result<impl Responder, BackendError> {
	body.validate()?;

	let Some(user_id) = identity.has_scope(Scope::Servers(ReadWrite::Write)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let server_id = path.into_inner();

	if !query!(
		"SELECT EXISTS(SELECT 1 FROM Server WHERE id = ? AND owner_id = ?) AS `exists: bool`",
		server_id,
		user_id
	)
	.fetch_one(&app_state.db)
	.await?
	.exists
	{
		return Ok(HttpResponse::NotFound().finish());
	};

	if query!(
		"SELECT COUNT(*) > 200 AS `over_limit: bool` FROM Channel WHERE server_id = ?",
		server_id
	)
	.fetch_one(&app_state.db)
	.await?
	.over_limit
	{
		return Ok(HttpResponse::BadRequest().json(ErrorResponse {
			error: "channel_limit_reached".to_string(),
		}));
	}

	let channel_id = {
		let mut generator = generator.lock().unwrap();
		generator.generate()
	};

	query!(
		"INSERT INTO Channel (id, name, kind, server_id) VALUES (?, ?, 'text', ?)",
		channel_id,
		body.name,
		server_id
	)
	.execute(&app_state.db)
	.await?;

	let channel = Channel {
		id: channel_id,
		name: body.name.to_string(),
		kind: ChannelKind::Text,
		server_id: Some(server_id),
		user: None,
	};

	if let Some(members) = app_state.server_connections.get(&server_id) {
		send_updates(
			[WsUpdateEvent::ChannelCreate(channel.clone())],
			&app_state,
			members.iter().copied(),
		);
	}

	Ok(HttpResponse::Created().json(channel))
}

pub async fn get_channels(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	path: web::Path<u64>,
) -> Result<impl Responder, BackendError> {
	let Some(user_id) = identity.has_scope(Scope::Servers(ReadWrite::Read)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let server_id = path.into_inner();

	if !query!(
        "SELECT EXISTS(SELECT 1 FROM ServerMember WHERE server_id = ? AND user_id = ?) AS `exists: bool`",
        server_id,
        user_id
    )
    .fetch_one(&app_state.db)
    .await?
        .exists
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

	let channels = query!(
		"SELECT id, name, kind FROM Channel WHERE server_id = ?",
		server_id
	)
	.fetch_all(&app_state.db)
	.await?;

	Ok(HttpResponse::Ok().json(
		channels
			.into_iter()
			.map(|row| Channel {
				id: row.id,
				name: row.name,
				kind: row.kind.parse().unwrap(),
				server_id: Some(server_id),
				user: None,
			})
			.collect::<Vec<_>>(),
	))
}

pub async fn get_channel(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	path: web::Path<(u64, u64)>,
) -> Result<impl Responder, BackendError> {
	let Some(user_id) = identity.has_scope(Scope::Servers(ReadWrite::Read)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let (server_id, channel_id) = path.into_inner();

	if !query!(
        "SELECT EXISTS(SELECT 1 FROM ServerMember WHERE server_id = ? AND user_id = ?) AS `exists: bool`",
        server_id,
        user_id
    )
    .fetch_one(&app_state.db)
    .await?
        .exists
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

	let channel = query!(
		"SELECT name, kind FROM Channel WHERE id = ? AND server_id = ?",
		channel_id,
		server_id
	)
	.fetch_optional(&app_state.db)
	.await?;

	Ok(HttpResponse::Ok().json(channel.map(|row| Channel {
		id: channel_id,
		name: row.name,
		kind: row.kind.parse().unwrap(),
		server_id: Some(server_id),
		user: None,
	})))
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateChannelBody {
	#[serde(default, deserialize_with = "super::trim_opt_string")]
	#[validate(length(min = 2, max = 32))]
	name: Option<String>,
}

pub async fn update_channel(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	body: web::Json<UpdateChannelBody>,
	path: web::Path<(u64, u64)>,
) -> Result<impl Responder, BackendError> {
	body.validate()?;

	let Some(user_id) = identity.has_scope(Scope::Servers(ReadWrite::Write)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let (server_id, channel_id) = path.into_inner();

	if !query!(
		"SELECT EXISTS(SELECT 1 FROM Server WHERE id = ? AND owner_id = ?) AS `exists: bool`",
		server_id,
		user_id
	)
	.fetch_one(&app_state.db)
	.await?
	.exists
	{
		return Ok(HttpResponse::Forbidden().finish());
	}

	let query = update_structure!("Channel", body, name)
		.push(" WHERE id = ")
		.push_bind(channel_id)
		.push(" AND server_id = ")
		.push_bind(server_id)
		.build()
		.execute(&app_state.db)
		.await?;

	if query.rows_affected() == 0 {
		return Ok(HttpResponse::NotFound().finish());
	}

	if let Some(members) = app_state.server_connections.get(&server_id) {
		send_updates(
			[WsUpdateEvent::ChannelUpdate {
				id: channel_id,
				name: body.name.clone(),
			}],
			&app_state,
			members.iter().copied(),
		);
	}

	Ok(HttpResponse::Ok().finish())
}

pub async fn delete_channel(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	path: web::Path<(u64, u64)>,
) -> Result<impl Responder, BackendError> {
	let Some(user_id) = identity.has_scope(Scope::Servers(ReadWrite::Write)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let (server_id, channel_id) = path.into_inner();

	if !query!(
		"SELECT EXISTS(SELECT 1 FROM Server WHERE id = ? AND owner_id = ?) AS `exists: bool`",
		server_id,
		user_id
	)
	.fetch_one(&app_state.db)
	.await?
	.exists
	{
		return Ok(HttpResponse::Forbidden().finish());
	}

	let query = query!(
		"DELETE FROM Channel WHERE id = ? AND server_id = ?",
		channel_id,
		server_id
	)
	.execute(&app_state.db)
	.await?;

	if query.rows_affected() == 0 {
		return Ok(HttpResponse::NotFound().finish());
	}

	if let Some(members) = app_state.server_connections.get(&server_id) {
		send_updates(
			[WsUpdateEvent::ChannelDelete { id: channel_id }],
			&app_state,
			members.iter().copied(),
		);
	}

	Ok(HttpResponse::Ok().finish())
}
