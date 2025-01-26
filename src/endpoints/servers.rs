use crate::{
	error::BackendError,
	middleware::Identity,
	models::{
		channel::{Channel, ChannelKind},
		scope::{HasScope, ReadWrite, Scope},
		server::Server,
		servermember::ServerMember,
	},
	update_structure,
	ws::{send_updates, WsUpdateEvent},
	AppState,
};
use actix_web::{web, HttpResponse, Responder};
use dashmap::mapref::entry::Entry;
use serde::Deserialize;
use serde_json::json;
use sqlx::query;
use std::sync::Mutex;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateServerBody {
	#[serde(deserialize_with = "super::trim_string")]
	#[validate(length(min = 2, max = 32))]
	name: String,
}

pub async fn create_server(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	generator: web::Data<Mutex<snowflaked::Generator>>,
	body: web::Json<CreateServerBody>,
) -> Result<impl Responder, BackendError> {
	body.validate()?;

	let Some(user_id) = identity.has_scope(Scope::Servers(ReadWrite::Write)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let (server_id, channel_id): (u64, u64) = {
		let mut generator = generator.lock().unwrap();
		(generator.generate(), generator.generate())
	};

	let mut tx = app_state.db.begin().await?;

	query!(
		"INSERT INTO Server (id, owner_id, name) VALUES (?, ?, ?)",
		server_id,
		user_id,
		body.name
	)
	.execute(&mut *tx)
	.await?;

	let created_at = chrono::Utc::now();

	query!(
        "INSERT INTO ServerMember (server_id, user_id, created_at, nickname) VALUES (?, ?, ?, NULL)",
        server_id,
        user_id,
        created_at
    )
    .execute(&mut *tx)
    .await?;

	query!(
		"INSERT INTO Channel (id, name, kind, server_id) VALUES (?, 'general', 'text', ?)",
		channel_id,
		server_id
	)
	.execute(&mut *tx)
	.await?;

	tx.commit().await?;

	let server = Server {
		id: server_id,
		name: body.name.clone(),
		owner_id: user_id,
	};

	let channel = Channel {
		id: channel_id,
		name: "general".to_string(),
		kind: ChannelKind::Text,
		server_id: Some(server_id),
		user: None,
	};

	let member = ServerMember {
		server_id,
		user_id,
		nickname: None,
		created_at,
		user: None,
	};

	if app_state
		.user_connections
		.get(&user_id)
		.is_some_and(|conns| !conns.is_empty())
	{
		app_state
			.server_connections
			.entry(server_id)
			.or_default()
			.insert(user_id);

		send_updates(
			[
				WsUpdateEvent::ServerCreate(server.clone()),
				WsUpdateEvent::MemberCreate(member.clone()),
				WsUpdateEvent::ChannelCreate(channel.clone()),
			],
			&app_state,
			[user_id],
		);
	}

	let mut value = serde_json::to_value(server).unwrap();
	value["channels"] = json!([channel]);
	value["members"] = json!([member]);

	Ok(HttpResponse::Created().json(value))
}

pub async fn get_servers(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
) -> Result<impl Responder, BackendError> {
	let Some(user_id) = identity.has_scope(Scope::Servers(ReadWrite::Read)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let servers = query!(
        "SELECT Server.id, Server.name, Server.owner_id FROM Server INNER JOIN ServerMember ON Server.id=ServerMember.server_id WHERE ServerMember.user_id = ?",
        user_id
    )
    .fetch_all(&app_state.db)
    .await?;

	Ok(HttpResponse::Ok().json(
        servers
            .into_iter()
            .map(|row| json!({ "id": row.id.to_string(), "name": row.name, "owner_id": row.owner_id.to_string() }))
            .collect::<Vec<_>>(),
    ))
}

pub async fn get_server(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	server_id: web::Path<u64>,
) -> Result<impl Responder, BackendError> {
	let Some(user_id) = identity.has_scope(Scope::Servers(ReadWrite::Read)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let server_id = server_id.into_inner();
	let server = query!(
        "SELECT Server.id, Server.name, Server.owner_id FROM Server INNER JOIN ServerMember ON Server.id=ServerMember.server_id WHERE ServerMember.user_id = ? AND Server.id = ?",
        user_id,
        server_id
    )
    .fetch_optional(&app_state.db)
    .await?;

	if let Some(server) = server {
		Ok(HttpResponse::Ok().json(json!({ "id": server.id.to_string(), "name": server.name, "owner_id": server.owner_id.to_string() })))
	} else {
		Ok(HttpResponse::NotFound().finish())
	}
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateServerBody {
	#[serde(default, deserialize_with = "super::trim_opt_string")]
	#[validate(length(min = 2, max = 32))]
	name: Option<String>,
}

pub async fn update_server(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	server_id: web::Path<u64>,
	body: web::Json<UpdateServerBody>,
) -> Result<impl Responder, BackendError> {
	body.validate()?;
	let Some(user_id) = identity.has_scope(Scope::Servers(ReadWrite::Write)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let server_id = server_id.into_inner();

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

	update_structure!("Server", body, name)
		.push(" WHERE id = ")
		.push_bind(server_id)
		.build()
		.execute(&app_state.db)
		.await?;

	if let Some(members) = app_state.server_connections.get(&server_id) {
		send_updates(
			[WsUpdateEvent::ServerUpdate {
				id: server_id,
				name: body.name.clone(),
			}],
			&app_state,
			members.iter().copied(),
		);
	}

	Ok(HttpResponse::Ok().finish())
}

pub async fn leave_server(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	server_id: web::Path<u64>,
) -> Result<impl Responder, BackendError> {
	let Some(user_id) = identity.has_scope(Scope::Servers(ReadWrite::Write)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let server_id = server_id.into_inner();

	let Some(server_record) = query!("SELECT Server.owner_id FROM ServerMember INNER JOIN Server ON ServerMember.server_id=Server.id WHERE ServerMember.user_id = ? AND ServerMember.server_id = ?", user_id, server_id)
        .fetch_optional(&app_state.db)
        .await?
        else
    {
        return Ok(HttpResponse::NotFound().finish());
    };

	if server_record.owner_id == user_id {
		return Ok(HttpResponse::Forbidden().finish());
	}

	query!(
		"DELETE FROM ServerMember WHERE server_id = ? AND user_id = ?",
		server_id,
		user_id
	)
	.execute(&app_state.db)
	.await?;

	send_updates(
		[WsUpdateEvent::ServerDelete { id: server_id }],
		&app_state,
		// notify the user that they left the server
		[user_id],
	);

	if let Entry::Occupied(mut members) = app_state.server_connections.entry(server_id) {
		send_updates(
			[WsUpdateEvent::MemberDelete { server_id, user_id }],
			&app_state,
			members.get().iter().copied(),
		);

		if members.get().len() == 1 {
			members.remove_entry();
		} else {
			members.get_mut().remove(&user_id);
		}
	}

	Ok(HttpResponse::Ok().finish())
}

pub async fn delete_server(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	server_id: web::Path<u64>,
) -> Result<impl Responder, BackendError> {
	let Some(user_id) = identity.has_scope(Scope::Servers(ReadWrite::Write)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let server_id = server_id.into_inner();
	let query = query!(
		"DELETE FROM Server WHERE id = ? AND owner_id = ?",
		server_id,
		user_id
	)
	.execute(&app_state.db)
	.await?;

	if query.rows_affected() == 0 {
		return Ok(HttpResponse::Forbidden().finish());
	}

	if let Entry::Occupied(members) = app_state.server_connections.entry(server_id) {
		send_updates(
			[WsUpdateEvent::ServerDelete { id: server_id }],
			&app_state,
			members.get().iter().copied(),
		);
		members.remove_entry();
	}

	Ok(HttpResponse::Ok().finish())
}
