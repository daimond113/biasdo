use std::{collections::HashSet, hash::Hash};

use actix_governor::{KeyExtractor, SimpleKeyExtractionError};
use actix_web::{
	body::MessageBody,
	dev::{ServiceRequest, ServiceResponse},
	http::header::AUTHORIZATION,
	middleware::Next,
	web, Error as ActixError, HttpMessage, HttpResponse, ResponseError,
};
use base64::Engine;
use sqlx::query;

use crate::{error::BackendError, models::scope::Scope, AppState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identity {
	User(u64),
	Client(u64),
	// Bearer tokens
	// refers to the fact a client is acting on behalf of a user, not the user itself
	UserByClient((u64, HashSet<Scope>)),
	ClientByClient((u64, HashSet<Scope>)),
}

impl Hash for Identity {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		match self {
			Identity::User(id) => {
				id.hash(state);
			}
			Identity::Client(id) => {
				id.hash(state);
			}
			Identity::UserByClient((id, scopes)) => {
				id.hash(state);
				for scope in scopes {
					scope.hash(state);
				}
			}
			Identity::ClientByClient((id, scopes)) => {
				id.hash(state);
				for scope in scopes {
					scope.hash(state);
				}
			}
		}
	}
}

impl Identity {
	pub fn is_user_like(&self) -> bool {
		matches!(self, Identity::User(_) | Identity::UserByClient(_))
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token(pub String);

async fn basic_token(
	token: &str,
	app_state: &web::Data<AppState>,
) -> Result<Option<Identity>, BackendError> {
	let token = match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(token.as_bytes()) {
		Ok(token) => token,
		Err(_) => return Ok(None),
	};
	let (client_id, client_secret) = match std::str::from_utf8(&token) {
		Ok(token) => match token.split_once(':') {
			Some(credentials) => credentials,
			None => return Ok(None),
		},
		Err(_) => return Ok(None),
	};

	if !query!(
		"SELECT EXISTS(SELECT 1 FROM Client WHERE id = ? AND secret = ?) AS `exists: bool`",
		client_id,
		client_secret
	)
	.fetch_one(&app_state.db)
	.await?
	.exists
	{
		return Ok(None);
	}

	Ok(Some(Identity::Client(client_id.parse().unwrap())))
}

fn scopes_from_string(scopes: &str) -> HashSet<Scope> {
	scopes
		.split(',')
		.filter(|s| !s.is_empty())
		.map(|s| s.parse().unwrap())
		.collect()
}

async fn bearer_token(
	token: &str,
	app_state: &web::Data<AppState>,
) -> Result<Option<Identity>, BackendError> {
	if token.starts_with("u.") {
		let Some(record) = query!(
            "SELECT user_id, scope FROM ClientUserTokens WHERE access_token = ? AND access_expires_at > NOW() AND expires_at > NOW()",
            token
        )
            .fetch_optional(&app_state.db)
            .await? else {
                return Ok(None);
            };

		Ok(Some(Identity::UserByClient((
			record.user_id,
			scopes_from_string(&record.scope),
		))))
	} else {
		let Some(record) = query!(
            "SELECT client_id, scope FROM ClientToken WHERE access_token = ? AND expires_at > NOW()",
            token
        )
            .fetch_optional(&app_state.db)
            .await? else {
                return Ok(None);
            };

		Ok(Some(Identity::ClientByClient((
			record.client_id,
			scopes_from_string(&record.scope),
		))))
	}
}

async fn other_token(
	token: &str,
	app_state: &web::Data<AppState>,
) -> Result<Option<Identity>, BackendError> {
	let Some(session_record) = query!(
		"SELECT user_id FROM UserSession WHERE id = ? AND expires_at > NOW()",
		token
	)
	.fetch_optional(&app_state.db)
	.await?
	else {
		return Ok(None);
	};

	query!(
        "UPDATE UserSession INNER JOIN User ON UserSession.user_id=User.id SET UserSession.expires_at = DEFAULT, User.began_deletion_at = NULL WHERE UserSession.id = ?",
        token
    )
    .execute(&app_state.db)
    .await?;

	Ok(Some(Identity::User(session_record.user_id)))
}

pub async fn get_identity(
	token: &str,
	app_state: &web::Data<AppState>,
) -> Result<Option<(Identity, Token)>, BackendError> {
	let identity = if let Some(token) = token.strip_prefix("Basic ") {
		basic_token(token, app_state).await?
	} else if let Some(token) = token.strip_prefix("Bearer ") {
		bearer_token(token, app_state).await?
	} else {
		other_token(token, app_state).await?
	};

	match identity {
		Some(identity) => Ok(Some((identity, Token(token.to_string())))),
		None => Ok(None),
	}
}

async fn get_identity_from_req(
	req: &ServiceRequest,
	app_state: &web::Data<AppState>,
) -> Result<Option<Option<(Identity, Token)>>, BackendError> {
	let Some(token) = req
		.headers()
		.get(AUTHORIZATION)
		.and_then(|t| t.to_str().ok())
	else {
		return Ok(None);
	};

	get_identity(token, app_state).await.map(Some)
}

pub async fn authentication(
	app_state: web::Data<AppState>,
	req: ServiceRequest,
	next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, ActixError> {
	let (identity, token) = match get_identity_from_req(&req, &app_state).await {
		Ok(identity) => match identity {
			Some(Some(identity)) => identity,
			_ => {
				return Ok(req
					.into_response(HttpResponse::Unauthorized().finish())
					.map_into_right_body())
			}
		},
		Err(e) => return Ok(req.into_response(e.error_response()).map_into_right_body()),
	};

	req.extensions_mut().insert(identity);
	req.extensions_mut().insert(token);

	let res = next.call(req).await?;
	Ok(res.map_into_left_body())
}

pub async fn maybe_authentication(
	app_state: web::Data<AppState>,
	req: ServiceRequest,
	next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, ActixError> {
	let identity = match get_identity_from_req(&req, &app_state).await {
		Ok(identity) => match identity {
			Some(None) => {
				return Ok(req
					.into_response(HttpResponse::Unauthorized().finish())
					.map_into_right_body())
			}
			o => o.flatten(),
		},
		Err(e) => return Ok(req.into_response(e.error_response()).map_into_right_body()),
	};

	let (identity, token) = match identity {
		Some((identity, token)) => (Some(identity), Some(token)),
		None => (None, None),
	};

	req.extensions_mut().insert(identity);
	req.extensions_mut().insert(token);

	let res = next.call(req).await?;
	Ok(res.map_into_left_body())
}

#[derive(Debug, Clone, Copy)]
pub struct TokenKey;

impl KeyExtractor for TokenKey {
	type Key = Token;
	type KeyExtractionError = SimpleKeyExtractionError<&'static str>;

	fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
		match req.extensions().get::<Token>() {
			Some(token) => Ok(token.clone()),
			None => Err(SimpleKeyExtractionError::new(
				"expected token in request extensions",
			)),
		}
	}
}
