use crate::{
	endpoints::oauth::{CodeChallengeMethod, ErrorResponse},
	error::Error,
	middleware::Identity,
	models::scope::Scope,
	AppState,
};
use actix_web::{web, HttpResponse, Responder};
use base64::Engine;
use cuid2::CuidConstructor;
use serde::{Deserialize, Serialize};
use serde_with::{formats::SpaceSeparator, serde_as, StringWithSeparator};
use sha2::{Digest, Sha256};
use sqlx::query;
use std::{collections::HashSet, sync::LazyLock};

#[serde_as]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "grant_type")]
pub enum GrantType {
	AuthorizationCode {
		code: String,
		code_verifier: Option<String>,
		client_id: Option<u64>,
	},
	RefreshToken {
		refresh_token: String,
		#[serde_as(as = "Option<StringWithSeparator<SpaceSeparator, Scope>>")]
		scope: Option<HashSet<Scope>>,
		client_id: Option<u64>,
	},
	ClientCredentials {
		#[serde_as(as = "Option<StringWithSeparator<SpaceSeparator, Scope>>")]
		scope: Option<HashSet<Scope>>,
	},
}

static TOKEN_GENERATOR: LazyLock<CuidConstructor> =
	LazyLock::new(|| CuidConstructor::new().with_length(64));

#[derive(Debug, Serialize)]
struct TokenResponse {
	access_token: String,
	token_type: &'static str,
	expires_in: u16,
	refresh_token: Option<String>,
	scope: String,
}

fn get_client_id(client_id: Option<u64>, identity: Option<Identity>) -> Result<u64, HttpResponse> {
	match (client_id, identity) {
		(Some(id), None) => Ok(id),
		(None, Some(identity)) => match identity {
			Identity::Client((id, None)) => Ok(id),
			_ => Err(HttpResponse::Forbidden().finish()),
		},
		(Some(_), Some(_)) => Err(HttpResponse::BadRequest().json(ErrorResponse {
			redirect: false,
			error: "invalid_request",
			error_description: "May not use client_id with authorization header",
		})),
		(None, None) => Err(HttpResponse::BadRequest().json(ErrorResponse {
			redirect: false,
			error: "invalid_request",
			error_description: "client_id or authorization header is required",
		})),
	}
}

pub async fn exchange_token(
	app_state: web::Data<AppState>,
	body: web::Form<GrantType>,
	identity: web::ReqData<Option<Identity>>,
) -> Result<impl Responder, Error> {
	let identity = identity.into_inner();

	match body.into_inner() {
		GrantType::AuthorizationCode {
			code,
			code_verifier,
			client_id,
		} => {
			let client_id = match get_client_id(client_id, identity) {
				Ok(id) => id,
				Err(resp) => return Ok(resp),
			};

			let Some(record) = query!(
                "SELECT user_id, scope, code_challenge, code_challenge_method FROM AuthorizationCode WHERE id = ? AND client_id = ? AND expires_at > NOW()",
                code,
                client_id
            )
            .fetch_optional(&app_state.db)
            .await? else {
                return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                    redirect: false,
                    error: "invalid_grant",
                    error_description: "Invalid authorization code"
                }));
            };

			let deleted_rows = query!("DELETE ClientUserTokens FROM ClientUserTokens INNER JOIN AuthorizationCode ON AuthorizationCode.id=ClientUserTokens.auth_code WHERE ClientUserTokens.auth_code = ? AND AuthorizationCode.expires_at > NOW()", code)
                .execute(&app_state.db)
                .await?;

			if deleted_rows.rows_affected() > 0 {
				return Ok(HttpResponse::BadRequest().json(ErrorResponse {
					redirect: false,
					error: "invalid_grant",
					error_description: "Invalid authorization code",
				}));
			}

			let verified = match (record.code_challenge, code_verifier) {
				(Some(challenge), Some(verifier)) => {
					let code_challenge_method = record.code_challenge_method.parse().unwrap();

					match code_challenge_method {
						CodeChallengeMethod::Plain => challenge == verifier,
						CodeChallengeMethod::S256 => {
							let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
								.decode(challenge.as_bytes())
								.unwrap();

							let mut sha = Sha256::default();
							sha.update(verifier.as_bytes());
							let verifier = sha.finalize();

							challenge == verifier.to_vec()
						}
					}
				}
				(None, None) => true,
				_ => {
					return Ok(HttpResponse::BadRequest().json(ErrorResponse {
						redirect: true,
						error: "invalid_request",
						error_description: "Code verifier mismatch",
					}))
				}
			};

			if !verified {
				return Ok(HttpResponse::BadRequest().json(ErrorResponse {
					redirect: false,
					error: "invalid_grant",
					error_description: "Invalid code verifier",
				}));
			}

			let access_token = format!("u.{}", TOKEN_GENERATOR.create_id());
			let refresh_token = format!("u.{}", TOKEN_GENERATOR.create_id());

			// REPLACE so the user can re-authenticate without needing an extra query to delete the old token
			query!(
                "REPLACE INTO ClientUserTokens (user_id, client_id, created_at, access_expires_at, expires_at, auth_code, access_token, refresh_token, scope) VALUES (?, ?, DEFAULT, DEFAULT, DEFAULT, ?, ?, ?, ?)",
                record.user_id,
                client_id,
                code,
                access_token,
                refresh_token,
                record.scope
            )
            .execute(&app_state.db)
            .await?;

			Ok(HttpResponse::Ok().json(TokenResponse {
				access_token,
				token_type: "Bearer",
				expires_in: 600,
				refresh_token: Some(refresh_token),
				scope: record.scope.replace(',', " "),
			}))
		}
		GrantType::RefreshToken {
			refresh_token,
			scope,
			client_id,
		} => {
			let client_id = match get_client_id(client_id, identity) {
				Ok(id) => id,
				Err(resp) => return Ok(resp),
			};

			let Some(record) = query!(
                "SELECT user_id, scope FROM ClientUserTokens WHERE refresh_token = ? AND client_id = ? AND expires_at > NOW()",
                refresh_token,
                client_id
            )
            .fetch_optional(&app_state.db)
            .await?
            else {
                return Ok(HttpResponse::BadRequest().json(ErrorResponse {
                    redirect: false,
                    error: "invalid_grant",
                    error_description: "Invalid refresh token"
                }));
            };

			let original_scope = record
				.scope
				.split(',')
				.filter(|s| !s.is_empty())
				.map(|s| s.parse().unwrap())
				.collect::<HashSet<Scope>>();

			let new_scope = if let Some(scope) = scope {
				if !scope
					.difference(&original_scope)
					.collect::<HashSet<_>>()
					.is_empty()
				{
					return Ok(HttpResponse::BadRequest().json(ErrorResponse {
						redirect: true,
						error: "invalid_scope",
						error_description: "Requested scope is not a subset of the original scope",
					}));
				}

				scope
			} else {
				original_scope
			};

			let access_token = format!("u.{}", TOKEN_GENERATOR.create_id());
			let refresh_token = format!("u.{}", TOKEN_GENERATOR.create_id());

			query!(
                "UPDATE ClientUserTokens SET access_token = ?, refresh_token = ?, scope = ?, expires_at = DEFAULT, access_expires_at = DEFAULT WHERE user_id = ? AND client_id = ?",
                access_token,
                refresh_token,
                new_scope
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
                record.user_id,
                client_id
            )
            .execute(&app_state.db)
            .await?;

			Ok(HttpResponse::Ok().json(TokenResponse {
				access_token,
				token_type: "Bearer",
				expires_in: 600,
				refresh_token: Some(refresh_token),
				scope: new_scope
					.iter()
					.map(|s| s.to_string())
					.collect::<Vec<String>>()
					.join(" "),
			}))
		}
		GrantType::ClientCredentials { scope } => {
			let Some(client_id) = identity else {
				return Ok(HttpResponse::BadRequest().json(ErrorResponse {
					redirect: false,
					error: "invalid_request",
					error_description: "Must authenticate as a client",
				}));
			};

			let client_id = match client_id {
				Identity::Client((id, None)) => id,
				_ => {
					return Ok(HttpResponse::Forbidden().finish());
				}
			};

			if !query!(
				"SELECT EXISTS(SELECT 1 FROM Client WHERE id = ?) AS `exists: bool`",
				client_id
			)
			.fetch_one(&app_state.db)
			.await?
			.exists
			{
				return Ok(HttpResponse::BadRequest().json(ErrorResponse {
					redirect: false,
					error: "invalid_client",
					error_description: "Invalid client",
				}));
			}

			let access_token = format!("c.{}", TOKEN_GENERATOR.create_id());

			let scope = match scope {
				Some(scope) => scope
					.iter()
					.map(|s| s.to_string())
					.collect::<Vec<String>>()
					.join(","),
				None => "".to_string(),
			};

			query!(
                "INSERT INTO ClientToken (access_token, client_id, created_at, expires_at, scope) VALUES (?, ?, DEFAULT, DEFAULT, ?)",
                access_token,
                client_id,
                scope
            )
            .execute(&app_state.db)
            .await?;

			Ok(HttpResponse::Ok().json(TokenResponse {
				access_token,
				token_type: "Bearer",
				expires_in: 600,
				refresh_token: None,
				scope: scope.replace(',', " "),
			}))
		}
	}
}
