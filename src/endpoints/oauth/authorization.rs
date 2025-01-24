use std::collections::HashSet;

use actix_web::{web, HttpResponse, Responder};
use cuid2::CuidConstructor;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json::json;
use serde_with::{formats::SpaceSeparator, serde_as, StringWithSeparator};
use sqlx::query;
use url::Url;

use crate::{
	endpoints::oauth::{CodeChallengeMethod, ErrorResponse},
	error::Error,
	middleware::Identity,
	models::{client::Client, scope::Scope},
	AppState,
};

static CODE_GENERATOR: Lazy<CuidConstructor> = Lazy::new(|| CuidConstructor::new().with_length(32));

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct AuthorizeQuery {
	client_id: u64,
	redirect_uri: Url,
	response_type: String,
	#[serde_as(as = "StringWithSeparator<SpaceSeparator, Scope>")]
	scope: HashSet<Scope>,
	state: Option<String>,
	code_challenge: Option<String>,
	code_challenge_method: Option<CodeChallengeMethod>,
}

// this differs from get_client because it doesn't require the user to be the owner of the client,
// but keeps potentially sensitive information (redirect_uris) hidden
// it also validates the query, so the website can display an error message before the "Authorize" button is clicked
pub async fn get_authorize_info(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	query: web::Query<AuthorizeQuery>,
) -> Result<impl Responder, Error> {
	if !matches!(identity.into_inner(), Identity::User(_)) {
		return Ok(HttpResponse::Forbidden().finish());
	}

	if query.response_type != "code" {
		return Ok(HttpResponse::BadRequest().json(ErrorResponse {
			redirect: true,
			error: "unsupported_response_type",
			error_description: "Response type is not supported",
		}));
	}

	let Some(client) = query!(
        "SELECT Client.id, Client.name, Client.owner_id, Client.client_uri, Client.tos_uri, Client.policy_uri FROM Client INNER JOIN ClientRedirect ON ClientRedirect.client_id=Client.id WHERE Client.id = ? AND ClientRedirect.uri = ?",
        query.client_id,
        query.redirect_uri.as_str()
    )
    .fetch_optional(&app_state.db)
    .await? else {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            redirect: false,
            error: "invalid_client",
            error_description: "Client does not exist or redirect URI is not registered"
        }));
    };

	let client = Client {
		id: client.id,
		name: client.name,
		client_uri: client.client_uri.map(|u| u.parse().unwrap()),
		tos_uri: client.tos_uri.map(|u| u.parse().unwrap()),
		policy_uri: client.policy_uri.map(|u| u.parse().unwrap()),
		owner_id: client.owner_id,
		redirect_uris: vec![],
	};

	Ok(HttpResponse::Ok().json(client))
}

pub async fn authorize_client(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	query: Result<web::Query<AuthorizeQuery>, actix_web::Error>,
) -> Result<impl Responder, Error> {
	let Ok(web::Query(AuthorizeQuery {
		client_id,
		scope,
		redirect_uri,
		response_type,
		state,
		code_challenge,
		code_challenge_method,
	})) = query
	else {
		return Ok(HttpResponse::BadRequest().json(ErrorResponse {
			redirect: true,
			error: "invalid_request",
			error_description: "Invalid query parameters",
		}));
	};

	if response_type != "code" {
		return Ok(HttpResponse::BadRequest().json(ErrorResponse {
			redirect: true,
			error: "unsupported_response_type",
			error_description: "Response type is not supported",
		}));
	}

	let user_id = match identity.into_inner() {
		Identity::User((id, None)) => id,
		_ => return Ok(HttpResponse::Forbidden().finish()),
	};

	if !query!(
        "SELECT EXISTS(SELECT 1 FROM Client INNER JOIN ClientRedirect ON ClientRedirect.client_id=Client.id WHERE Client.id = ? AND ClientRedirect.uri = ?) AS `exists: bool`",
        client_id,
        redirect_uri.as_str()
    )
    .fetch_one(&app_state.db)
    .await?
    .exists
    {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            redirect: false,
            error: "invalid_client",
            error_description: "Client does not exist or redirect URI is not registered"
        }));
    };

	let code = CODE_GENERATOR.create_id();

	query!(
        "INSERT INTO AuthorizationCode (id, created_at, expires_at, client_id, user_id, scope, code_challenge, code_challenge_method) VALUES (?, DEFAULT, DEFAULT, ?, ?, ?, ?, ?)",
        code,
        client_id,
        user_id,
        scope
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(","),
        code_challenge,
        code_challenge_method
            .map(|m| m.to_string())
            .unwrap_or_else(|| "plain".to_string()),
    )
    .execute(&app_state.db)
    .await?;

	Ok(HttpResponse::Ok().json(json!({
		"code": code,
		"state": state,
	})))
}
