use actix_web::{web, HttpResponse, Responder};
use cuid2::CuidConstructor;
use once_cell::sync::Lazy;
use sqlx::{mysql::MySqlDatabaseError, query};

use crate::{
    error::{Error, ErrorResponse},
    middleware::Identity,
    models::{
        channel::Channel,
        invite::Invite,
        scope::{HasScope, ReadWrite, Scope},
        server::Server,
        servermember::ServerMember,
        user::User,
    },
    ws::{send_updates, WsUpdateEvent},
    AppState,
};

static INVITE_GENERATOR: Lazy<CuidConstructor> =
    Lazy::new(|| CuidConstructor::new().with_length(24));

pub async fn create_invite(
    identity: web::ReqData<Identity>,
    app_state: web::Data<AppState>,
    path: web::Path<u64>,
) -> Result<impl Responder, Error> {
    let Some(user_id) = identity.has_scope(Scope::Servers(ReadWrite::Write)) else {
        return Ok(HttpResponse::Forbidden().finish());
    };

    let server_id = path.into_inner();

    let Some(server) = query!(
        "SELECT name FROM Server WHERE id = ? AND owner_id = ?",
        server_id,
        user_id
    )
    .fetch_optional(&app_state.db)
    .await?
    else {
        return Ok(HttpResponse::Forbidden().finish());
    };

    if query!(
        "SELECT COUNT(*) > 30 AS `over_limit: bool` FROM ServerInvite WHERE server_id = ? AND expires_at > NOW()",
        server_id
    )
    .fetch_one(&app_state.db)
    .await?
    .over_limit
    {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "invite_limit_reached".to_string(),
        }));
    }

    let invite_code = INVITE_GENERATOR.create_id();
    let created_at = chrono::Utc::now();
    let expires_at = created_at + chrono::Duration::days(7);

    query!(
        "INSERT INTO ServerInvite (id, created_at, expires_at, server_id) VALUES (?, ?, ?, ?)",
        invite_code,
        created_at,
        expires_at,
        server_id
    )
    .execute(&app_state.db)
    .await?;

    let invite = Invite {
        id: invite_code.to_string(),
        created_at,
        expires_at,
        server: Server {
            id: server_id,
            name: server.name,
            owner_id: user_id,
        },
    };

    if let Some(members) = app_state.server_connections.get(&server_id) {
        send_updates(
            [WsUpdateEvent::InviteCreate(invite.clone())],
            &app_state,
            members.iter().copied(),
        );
    }

    Ok(HttpResponse::Ok().json(invite))
}

pub async fn get_invites(
    identity: web::ReqData<Identity>,
    app_state: web::Data<AppState>,
    path: web::Path<u64>,
) -> Result<impl Responder, Error> {
    if identity
        .has_scope(Scope::Servers(ReadWrite::Read))
        .is_none()
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let server_id = path.into_inner();

    let Some(server) = query!("SELECT name, owner_id FROM Server WHERE id = ?", server_id)
        .fetch_optional(&app_state.db)
        .await?
    else {
        return Ok(HttpResponse::Forbidden().finish());
    };

    let invites = query!(
        "SELECT id, created_at, expires_at FROM ServerInvite WHERE server_id = ? AND expires_at > NOW()",
        server_id
    )
    .fetch_all(&app_state.db)
    .await?;

    Ok(HttpResponse::Ok().json(
        invites
            .into_iter()
            .map(|row| Invite {
                id: row.id,
                created_at: row.created_at,
                expires_at: row.expires_at,
                server: Server {
                    id: server_id,
                    name: server.name.to_string(),
                    owner_id: server.owner_id,
                },
            })
            .collect::<Vec<_>>(),
    ))
}

pub async fn get_invite(
    identity: web::ReqData<Identity>,
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    if identity
        .has_scope(Scope::Servers(ReadWrite::Read))
        .is_none()
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let invite_id = path.into_inner();

    let Some(invite) = query!(
        "SELECT ServerInvite.id, ServerInvite.created_at, ServerInvite.expires_at, Server.id AS `server_id`, Server.owner_id, Server.name FROM ServerInvite INNER JOIN Server ON Server.id=ServerInvite.server_id WHERE ServerInvite.id = ? AND ServerInvite.expires_at > NOW()",
        invite_id
    )
    .fetch_optional(&app_state.db)
    .await? else {
        return Ok(HttpResponse::NotFound().finish());
    };

    Ok(HttpResponse::Ok().json(Invite {
        id: invite.id,
        created_at: invite.created_at,
        expires_at: invite.expires_at,
        server: Server {
            id: invite.server_id,
            name: invite.name,
            owner_id: invite.owner_id,
        },
    }))
}

pub async fn delete_invite(
    identity: web::ReqData<Identity>,
    app_state: web::Data<AppState>,
    path: web::Path<(u64, String)>,
) -> Result<impl Responder, Error> {
    let Some(user_id) = identity.has_scope(Scope::Servers(ReadWrite::Write)) else {
        return Ok(HttpResponse::Forbidden().finish());
    };

    let (server_id, invite_id) = path.into_inner();

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

    let results = query!(
        "DELETE FROM ServerInvite WHERE id = ? AND server_id = ? AND expires_at > NOW()",
        invite_id,
        server_id
    )
    .execute(&app_state.db)
    .await?;

    if results.rows_affected() == 0 {
        return Ok(HttpResponse::NotFound().finish());
    }

    if let Some(members) = app_state.server_connections.get(&server_id) {
        send_updates(
            [WsUpdateEvent::InviteDelete { id: invite_id }],
            &app_state,
            members.iter().copied(),
        );
    }

    Ok(HttpResponse::Ok().finish())
}

pub async fn accept_invite(
    identity: web::ReqData<Identity>,
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<impl Responder, Error> {
    let Some(user_id) = identity.has_scope(Scope::Servers(ReadWrite::Write)) else {
        return Ok(HttpResponse::Forbidden().finish());
    };

    let invite_id = path.into_inner();

    let Some(invite) = query!(
        "SELECT server_id, expires_at FROM ServerInvite WHERE id = ? AND expires_at > NOW()",
        invite_id
    )
    .fetch_optional(&app_state.db)
    .await?
    else {
        return Ok(HttpResponse::NotFound().finish());
    };

    let user = query!(
        "SELECT username, display_name FROM User WHERE id = ?",
        user_id
    )
    .fetch_one(&app_state.db)
    .await?;

    let created_at = chrono::Utc::now();

    let result = query!(
        "INSERT INTO ServerMember (server_id, user_id, created_at, nickname) VALUES (?, ?, ?, NULL)",
        invite.server_id,
        user_id,
        created_at
    )
    .execute(&app_state.db)
    .await;

    match result {
        Ok(_) => {
            if app_state
                .user_connections
                .get(&user_id)
                .is_some_and(|conns| !conns.is_empty())
            {
                app_state
                    .server_connections
                    .entry(invite.server_id)
                    .or_default()
                    .insert(user_id);

                let records = query!("SELECT Server.name, Server.owner_id, Channel.id, Channel.name AS `channel_name`, Channel.kind FROM Server LEFT JOIN Channel ON Server.id=Channel.server_id WHERE Server.id = ?", invite.server_id)
                     .fetch_all(&app_state.db)
                     .await?;

                send_updates(
                    std::iter::once(WsUpdateEvent::ServerCreate(Server {
                        id: invite.server_id,
                        name: records[0].name.clone(),
                        owner_id: records[0].owner_id,
                    }))
                    .chain(records.into_iter().filter_map(|row| {
                        match (row.id, row.channel_name, row.kind) {
                            (Some(id), Some(name), Some(kind)) => {
                                Some(WsUpdateEvent::ChannelCreate(Channel {
                                    id,
                                    name,
                                    server_id: Some(invite.server_id),
                                    kind: kind.parse().unwrap(),
                                    user: None,
                                }))
                            }
                            _ => None,
                        }
                    })),
                    &app_state,
                    [user_id],
                );
            }

            if let Some(members) = app_state.server_connections.get(&invite.server_id) {
                send_updates(
                    [WsUpdateEvent::MemberCreate(ServerMember {
                        server_id: invite.server_id,
                        user_id,
                        nickname: None,
                        created_at,
                        user: Some(User {
                            id: user_id,
                            username: user.username,
                            display_name: user.display_name,
                        }),
                    })],
                    &app_state,
                    members.iter().copied(),
                );
            }

            Ok(HttpResponse::Ok().finish())
        }
        Err(err) => match err.as_database_error() {
            Some(err)
                if err
                    .try_downcast_ref::<MySqlDatabaseError>()
                    .is_some_and(|err| err.number() == 1062) =>
            {
                Ok(HttpResponse::Conflict().json(ErrorResponse {
                    error: "You are already a member of this server".to_string(),
                }))
            }
            _ => Err(err.into()),
        },
    }
}
