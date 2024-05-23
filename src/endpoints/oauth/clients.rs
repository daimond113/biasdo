use std::{collections::HashMap, sync::Mutex};

use actix_web::{web, HttpResponse, Responder};
use cuid2::CuidConstructor;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::Value;
use sqlx::query;
use url::Url;
use validator::Validate;

use crate::{
    error::Error, middleware::Identity, models::client::Client, update_structure, AppState,
};

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ClientType {
    Public,
    #[default]
    Confidential,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterClientBody {
    #[validate(length(min = 2, max = 24))]
    name: String,
    client_uri: Option<Url>,
    tos_uri: Option<Url>,
    policy_uri: Option<Url>,
    #[validate(length(min = 1, max = 16))]
    redirect_uris: Vec<Url>,
    #[serde(default)]
    client_type: ClientType,
}

static CLIENT_SECRET_GENERATOR: Lazy<CuidConstructor> =
    Lazy::new(|| CuidConstructor::new().with_length(64));

pub async fn register_client(
    identity: web::ReqData<Identity>,
    generator: web::Data<Mutex<snowflaked::Generator>>,
    app_state: web::Data<AppState>,
    body: web::Json<RegisterClientBody>,
) -> Result<impl Responder, Error> {
    body.validate()?;

    let user_id = match identity.into_inner() {
        Identity::User((id, None)) => id,
        _ => return Ok(HttpResponse::Forbidden().finish()),
    };

    let client_id: u64 = {
        let mut generator = generator.lock().unwrap();
        generator.generate()
    };

    let client_secret = if matches!(body.client_type, ClientType::Confidential) {
        Some(CLIENT_SECRET_GENERATOR.create_id())
    } else {
        None
    };

    let mut tx = app_state.db.begin().await?;

    query!(
        "INSERT INTO Client (id, name, secret, owner_id, client_uri, tos_uri, policy_uri) VALUES (?, ?, ?, ?, ?, ?, ?)",
        client_id,
        body.name,
        client_secret,
        user_id,
        body.client_uri.as_ref().map(Url::as_str),
        body.tos_uri.as_ref().map(Url::as_str),
        body.policy_uri.as_ref().map(Url::as_str)
    )
    .execute(&mut *tx)
    .await?;

    for redirect_uri in body.redirect_uris.iter() {
        query!(
            "INSERT INTO ClientRedirect (client_id, uri) VALUES (?, ?)",
            client_id,
            redirect_uri.as_str()
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    let mut client = serde_json::to_value(Client {
        id: client_id,
        name: body.name.clone(),
        client_uri: body.client_uri.clone(),
        tos_uri: body.tos_uri.clone(),
        policy_uri: body.policy_uri.clone(),
        owner_id: user_id,
        redirect_uris: body.redirect_uris.clone(),
    })
    .unwrap();

    if let Some(secret) = client_secret {
        client["secret"] = Value::String(secret);
    }

    Ok(HttpResponse::Ok().json(client))
}

pub async fn get_clients(
    identity: web::ReqData<Identity>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, Error> {
    let user_id = match identity.into_inner() {
        Identity::User((id, None)) => id,
        _ => return Ok(HttpResponse::Forbidden().finish()),
    };

    let rows = query!(
        "SELECT Client.id, Client.name, Client.client_uri, Client.tos_uri, Client.policy_uri, ClientRedirect.uri FROM Client LEFT JOIN ClientRedirect ON ClientRedirect.client_id=Client.id WHERE Client.owner_id = ?",
        user_id
    )
    .fetch_all(&app_state.db)
    .await?;

    let mut clients = HashMap::new();

    for row in rows {
        let client_id = row.id;
        let client = clients.entry(client_id).or_insert_with(|| Client {
            id: client_id,
            name: row.name.clone(),
            client_uri: row.client_uri.clone().map(|u| u.parse().unwrap()),
            tos_uri: row.tos_uri.clone().map(|u| u.parse().unwrap()),
            policy_uri: row.policy_uri.clone().map(|u| u.parse().unwrap()),
            owner_id: user_id,
            redirect_uris: vec![],
        });

        if let Some(uri) = row.uri {
            client.redirect_uris.push(uri.parse().unwrap());
        }
    }

    Ok(HttpResponse::Ok().json(clients))
}

pub async fn get_client(
    identity: web::ReqData<Identity>,
    app_state: web::Data<AppState>,
    client_id: web::Path<u64>,
) -> Result<impl Responder, Error> {
    let user_id = match identity.into_inner() {
        Identity::User((id, None)) => id,
        _ => return Ok(HttpResponse::Forbidden().finish()),
    };

    let client = query!(
        "SELECT Client.id, Client.name, Client.owner_id, Client.client_uri, Client.tos_uri, Client.policy_uri, ClientRedirect.uri FROM Client LEFT JOIN ClientRedirect ON ClientRedirect.client_id=Client.id WHERE Client.id = ?",
        client_id.into_inner()
    )
        .fetch_all(&app_state.db)
        .await?;

    if client.is_empty() {
        return Ok(HttpResponse::NotFound().finish());
    }

    if client[0].owner_id != user_id {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let client = Client {
        id: client[0].id,
        name: client[0].name.clone(),
        client_uri: client[0].client_uri.clone().map(|u| u.parse().unwrap()),
        tos_uri: client[0].tos_uri.clone().map(|u| u.parse().unwrap()),
        policy_uri: client[0].policy_uri.clone().map(|u| u.parse().unwrap()),
        owner_id: user_id,
        redirect_uris: client
            .into_iter()
            .filter_map(|row| row.uri)
            .map(|u| u.parse().unwrap())
            .collect(),
    };

    Ok(HttpResponse::Ok().json(client))
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateClientBody {
    #[validate(length(min = 2, max = 24))]
    name: Option<String>,
    client_uri: Option<Url>,
    tos_uri: Option<Url>,
    policy_uri: Option<Url>,
    #[validate(length(min = 1, max = 16))]
    redirect_uris: Option<Vec<Url>>,
}

pub async fn update_client(
    identity: web::ReqData<Identity>,
    app_state: web::Data<AppState>,
    client_id: web::Path<u64>,
    body: web::Json<UpdateClientBody>,
) -> Result<impl Responder, Error> {
    body.validate()?;

    let user_id = match identity.into_inner() {
        Identity::User((id, None)) => id,
        _ => return Ok(HttpResponse::Forbidden().finish()),
    };

    let client_id = client_id.into_inner();

    if !query!(
        "SELECT EXISTS(SELECT 1 FROM Client WHERE id = ? AND owner_id = ?) AS `exists: bool`",
        client_id,
        user_id
    )
    .fetch_one(&app_state.db)
    .await?
    .exists
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let mut tx = app_state.db.begin().await?;

    let (mut pushed, mut query_builder) = update_structure!(raw "Client", body, name);

    if let Some(client_uri) = &body.client_uri {
        if pushed {
            query_builder.push(", ");
        }
        pushed = true;
        query_builder
            .push("client_uri = ")
            .push_bind(client_uri.as_str());
    }

    if let Some(tos_uri) = &body.tos_uri {
        if pushed {
            query_builder.push(", ");
        }
        pushed = true;
        query_builder.push("tos_uri = ").push_bind(tos_uri.as_str());
    }

    if let Some(policy_uri) = &body.policy_uri {
        if pushed {
            query_builder.push(", ");
        }
        pushed = true;
        query_builder
            .push("policy_uri = ")
            .push_bind(policy_uri.as_str());
    }

    if !pushed {
        return Ok(HttpResponse::BadRequest().finish());
    }

    query_builder
        .push(" WHERE id = ")
        .push_bind(client_id)
        .build()
        .execute(&mut *tx)
        .await?;

    if let Some(redirect_uris) = &body.redirect_uris {
        query!("DELETE FROM ClientRedirect WHERE client_id = ?", client_id)
            .execute(&mut *tx)
            .await?;

        for redirect_uri in redirect_uris {
            query!(
                "INSERT INTO ClientRedirect (client_id, uri) VALUES (?, ?)",
                client_id,
                redirect_uri.as_str()
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

pub async fn delete_client(
    identity: web::ReqData<Identity>,
    app_state: web::Data<AppState>,
    client_id: web::Path<u64>,
) -> Result<impl Responder, Error> {
    let user_id = match identity.into_inner() {
        Identity::User((id, None)) => id,
        _ => return Ok(HttpResponse::Forbidden().finish()),
    };

    let client_id = client_id.into_inner();

    if !query!(
        "SELECT EXISTS(SELECT 1 FROM Client WHERE id = ? AND owner_id = ?) AS `exists: bool`",
        client_id,
        user_id
    )
    .fetch_one(&app_state.db)
    .await?
    .exists
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    query!("DELETE FROM Client WHERE id = ?", client_id)
        .execute(&app_state.db)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}
