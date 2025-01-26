use actix_web::{web, HttpResponse};
use serde::Deserialize;
use sqlx::query;
use validator::Validate;

use crate::{
	error::ApiResult,
	middleware::Identity,
	models::{
		scope::{ReadWrite, Scope},
		servermember::ServerMember,
		user::User,
	},
	update_structure,
	ws::{send_updates, WsUpdateEvent},
	AppState,
};

#[derive(Debug, Deserialize, Validate)]
pub struct GetMembersQuery {
	#[validate(range(min = 1, max = 100))]
	limit: Option<u64>,
	last_id: Option<u64>,
}

macro_rules! member_row {
	($server_id:expr, $row:expr) => {{
		let user = User {
			id: $row.user_id,
			username: $row.username,
			display_name: $row.display_name,
		};

		ServerMember {
			user_id: $row.user_id,
			server_id: $server_id,
			nickname: $row.nickname,
			created_at: $row.created_at,
			user: Some(user),
		}
	}};
}

pub async fn get_members(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	path: web::Path<u64>,
	query: web::Query<GetMembersQuery>,
) -> ApiResult {
	query.validate()?;

	let Some(user_id) = identity.is_user_like_with_scope(Scope::Servers(ReadWrite::Read)) else {
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

	let limit = query.limit.unwrap_or(50);
	let last_id = query.last_id.unwrap_or(u64::MAX);

	let mut members = query!(
		r#"SELECT User.username, User.display_name,
ServerMember.nickname, ServerMember.created_at, ServerMember.user_id
FROM ServerMember
INNER JOIN User ON User.id=ServerMember.user_id
WHERE ServerMember.server_id = ? AND ServerMember.user_id < ?
ORDER BY id DESC
LIMIT ?
"#,
		server_id,
		last_id,
		limit
	)
	.fetch_all(&app_state.db)
	.await?;

	members.reverse();

	Ok(HttpResponse::Ok().json(
		members
			.into_iter()
			.map(|row| member_row!(server_id, row))
			.collect::<Vec<_>>(),
	))
}

pub async fn get_member(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	path: web::Path<(u64, u64)>,
) -> ApiResult {
	let Some(user_id) = identity.is_user_like_with_scope(Scope::Servers(ReadWrite::Read)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let (server_id, member_id) = path.into_inner();

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

	let Some(member) = query!(
		r#"SELECT User.username, User.display_name,
ServerMember.nickname, ServerMember.created_at, ServerMember.user_id
FROM ServerMember
INNER JOIN User ON User.id=ServerMember.user_id
WHERE ServerMember.server_id = ? AND ServerMember.user_id = ?
"#,
		server_id,
		member_id
	)
	.fetch_optional(&app_state.db)
	.await?
	else {
		return Ok(HttpResponse::NotFound().finish());
	};

	Ok(HttpResponse::Ok().json(member_row!(server_id, member)))
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMemberBody {
	#[validate(length(min = 2, max = 32))]
	#[serde(deserialize_with = "super::deserialize_some_trimmed")]
	nickname: Option<Option<String>>,
}

pub async fn update_member(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	path: web::Path<(u64, u64)>,
	body: web::Json<UpdateMemberBody>,
) -> ApiResult {
	body.validate()?;

	let Some(user_id) = identity.is_user_like_with_scope(Scope::Servers(ReadWrite::Write)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let (server_id, member_id) = path.into_inner();

	let Some(server) = query!(
        "SELECT Server.owner_id FROM Server INNER JOIN ServerMember ON ServerMember.server_id=Server.id AND ServerMember.user_id=? WHERE Server.id = ?",
        member_id,
        server_id
    )
    .fetch_optional(&app_state.db)
    .await? else {
        return Ok(HttpResponse::NotFound().finish());
    };

	// server owners can update any member, but other members can only update themselves
	if server.owner_id != user_id && member_id != user_id {
		return Ok(HttpResponse::Forbidden().finish());
	}

	let query = update_structure!("ServerMember", body, nickname)
		.push(" WHERE server_id = ")
		.push_bind(server_id)
		.push(" AND user_id = ")
		.push_bind(member_id)
		.build()
		.execute(&app_state.db)
		.await?;

	if query.rows_affected() == 0 {
		return Ok(HttpResponse::NotFound().finish());
	}

	if let Some(members) = app_state.server_connections.get(&server_id) {
		send_updates(
			[WsUpdateEvent::MemberUpdate {
				server_id,
				user_id: member_id,
				nickname: body.nickname.clone(),
			}],
			&app_state,
			members.iter().copied(),
		);
	}

	Ok(HttpResponse::Ok().finish())
}
