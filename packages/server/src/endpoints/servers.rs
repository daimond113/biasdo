use std::str::FromStr;

use actix_web::{get, HttpResponse, post, Responder, web};
use chrono::{DateTime, Utc};
use dashmap::DashSet;
use indexmap::IndexMap;
use serde::Deserialize;
use sqlx::{query, query_as};
use validator::Validate;

use crate::{
    AppState,
    consts::merge_json,
    errors::{self},
    structures::{
        self,
        channel::{Channel, ChannelKind},
        member::Member,
        server::Server,
        session::Session,
    },
};
use crate::ws::JsonMessage;

#[get("/servers/{id}")]
async fn get_server(
    data: web::Data<AppState>,
    session: web::ReqData<Session>,
    path: web::Path<u64>,
) -> Result<impl Responder, errors::Errors> {
    let server = query_as!(structures::server::Server, "SELECT id, created_at, name, owner_id FROM Server WHERE id = ? AND id IN (SELECT server_id FROM Member WHERE user_id = ?)", path.into_inner(), session.user_id)
        .fetch_optional(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    match server {
        Some(server) => {
            let mut server_value = serde_json::to_value(server.clone()).unwrap();
            let channels = query_as!(structures::channel::Channel, "SELECT id, created_at, name, kind AS `kind: _`, server_id FROM Channel WHERE server_id = ?", server.id)
                .fetch_all(&data.db)
                .await
                .map_err(errors::Errors::Db)?;

            merge_json(
                &mut server_value,
                &serde_json::json!({ "channels": channels }),
            );

            Ok(HttpResponse::Ok().json(server_value))
        }
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

#[get("/servers")]
async fn my_servers(
    data: web::Data<AppState>,
    session: web::ReqData<Session>,
) -> Result<impl Responder, errors::Errors> {
    let return_data = query!(
        "SELECT Server.id, Server.created_at, Server.name, Server.owner_id, Channel.id AS channel_id, Channel.created_at AS channel_created_at, Channel.name AS channel_name, Channel.kind AS channel_kind FROM Server JOIN Channel ON Server.id = Channel.server_id WHERE Server.id IN (SELECT server_id FROM Member WHERE user_id = ?) ORDER BY Server.id, Channel.id",
        session.user_id
    )
        .fetch_all(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    let mut server_channel_map: IndexMap<Server, Vec<Channel>> = IndexMap::new();

    return_data.iter().for_each(|row| {
        let server = Server {
            id: row.id.into(),
            created_at: row.created_at,
            name: row.name.clone(),
            owner_id: row.owner_id.into(),
        };

        let channel = Channel {
            id: row.channel_id.into(),
            created_at: row.channel_created_at,
            name: row.channel_name.clone(),
            kind: ChannelKind::from_str(row.channel_kind.as_str()).unwrap(),
            server_id: row.id.into(),
        };

        server_channel_map
            .entry(server)
            .or_insert_with(Vec::new)
            .push(channel);
    });

    Ok(HttpResponse::Ok().json(
        server_channel_map
            .iter()
            .map(|(server, channels)| {
                let mut server_value = serde_json::to_value(server.clone()).unwrap();
                merge_json(
                    &mut server_value,
                    &serde_json::json!({ "channels": channels }),
                );
                server_value
            })
            .collect::<Vec<_>>(),
    ))
}

#[derive(Deserialize, Validate)]
pub struct CreateServerData {
    #[validate(length(min = 2, max = 32))]
    name: String,
}

#[post("/servers")]
async fn create_server(
    req_body: web::Either<web::Json<CreateServerData>, web::Form<CreateServerData>>,
    data: web::Data<AppState>,
    session: web::ReqData<Session>,
) -> Result<impl Responder, errors::Errors> {
    let body = req_body.into_inner();
    body.validate().map_err(errors::Errors::Validation)?;

    let server: (u64, DateTime<Utc>) =
        query_as("INSERT INTO Server VALUES (NULL, DEFAULT, ?, ?) RETURNING id, created_at")
            .bind(body.name.clone())
            .bind(session.user_id)
            .fetch_one(&data.db)
            .await
            .map_err(errors::Errors::Db)?;

    let member: (u64, DateTime<Utc>) =
        query_as("INSERT INTO Member VALUES (NULL, DEFAULT, ?, ?, NULL) RETURNING id, created_at")
            .bind(server.0)
            .bind(session.user_id)
            .fetch_one(&data.db)
            .await
            .map_err(errors::Errors::Db)?;

    let channel: (u64, DateTime<Utc>) =
        query_as("INSERT INTO Channel VALUES (NULL, DEFAULT, ?, ?, ?) RETURNING id, created_at")
            .bind("general")
            .bind(ChannelKind::Text)
            .bind(server.0)
            .fetch_one(&data.db)
            .await
            .map_err(errors::Errors::Db)?;

    let member_struct = Member {
        id: member.0.into(),
        created_at: member.1,
        user_id: Some(session.user_id.0).into(),
        server_id: server.0.into(),
        nickname: None,
    };

    let channel_struct = Channel {
        id: channel.0.into(),
        created_at: channel.1,
        name: "general".to_string(),
        kind: ChannelKind::Text,
        server_id: server.0.into(),
    };

    let mut additional_data = serde_json::json!({
        "members": [member_struct],
        "channels": [channel_struct],
    });

    merge_json(
        &mut additional_data,
        &serde_json::to_value(Server {
            id: server.0.into(),
            created_at: server.1,
            name: body.name,
            owner_id: session.user_id,
        })
            .unwrap(),
    );

    let msg = JsonMessage(serde_json::json!({
        "type": "server_create",
        "data": additional_data.clone(),
    }));

    if let Some(owner_sockets) = data
        .user_connections
        .get(&session.user_id.0)
    {
        data.server_connections.entry(server.0).or_insert_with(DashSet::new).insert(session.user_id.0);
        
        owner_sockets.iter().for_each(|addr| {
            addr.do_send(msg.clone());
        })
    }

    Ok(HttpResponse::Ok().json(additional_data))
}
