use actix_web::cookie::time::Duration;
use actix_web::{cookie::{Cookie, CookieBuilder, SameSite}, post, web, HttpResponse, Responder, get};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, Utc};
use cuid2::create_id;
use serde::Deserialize;
use sqlx::{query, query_as};

use validator::Validate;

use crate::consts::SESSION_COOKIE_NAME;
use crate::structures::session::Session;
use crate::{
    errors::{self, remove_tokens_from_response, ErrorResponse},
    structures, AppState,
};

pub fn create_session_cookie(id: String, expires_at: DateTime<Utc>) -> Cookie<'static> {
    let chrono_duration = expires_at - Utc::now();
    CookieBuilder::new(SESSION_COOKIE_NAME, id)
        .path("/")
        .http_only(true)
        .same_site(SameSite::None)
        .secure(true)
        .max_age(Duration::milliseconds(chrono_duration.num_milliseconds()))
        .finish()
}

#[derive(Deserialize, Validate)]
pub struct LoginOrRegisterData {
    #[validate(length(min = 1, max = 16))]
    username: String,
    #[validate(length(min = 8, max = 70))]
    password: String,
    kind: Option<String>,
}

#[post("/auth")]
async fn auth(
    req_body: web::Either<web::Json<LoginOrRegisterData>, web::Form<LoginOrRegisterData>>,
    data: web::Data<AppState>,
) -> Result<impl Responder, errors::Errors> {
    let body = req_body.into_inner();
    body.validate().map_err(errors::Errors::Validation)?;

    let kind = match body.kind.unwrap_or("".to_string()).to_lowercase().as_str() {
        "login" => "login",
        _ => "register",
    };

    let user = query_as!(
        structures::user::User,
        "SELECT id, created_at, username, password FROM User WHERE username = ?",
        body.username
    )
    .fetch_optional(&data.db)
    .await
    .map_err(errors::Errors::Db)?;

    match kind {
        "login" => {
            if user.is_none()
                || !verify(body.password, user.as_ref().unwrap().password.as_str()).unwrap_or(false)
            {
                return Ok(HttpResponse::Unauthorized().json(ErrorResponse {
                    errors: "Incorrect username/password".to_string(),
                }));
            }

            let session: (String, DateTime<Utc>) = query_as(
                "INSERT INTO Session VALUES (?, DEFAULT, DEFAULT, ?) RETURNING id, expires_at",
            )
            .bind(create_id())
            .bind(user.unwrap().id)
            .fetch_one(&data.db)
            .await
            .map_err(errors::Errors::Db)?;

            Ok(HttpResponse::Ok()
                .cookie(create_session_cookie(session.0, session.1))
                .finish())
        }
        "register" => {
            if user.is_some() {
                return Ok(HttpResponse::Conflict().json(ErrorResponse {
                    errors: "Username already taken".to_string(),
                }));
            }

            let hashed = hash(body.password, DEFAULT_COST);
            if hashed.is_err() {
                return Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                    errors: "Unable to hash password".to_string(),
                }));
            }

            let user: (u64,) =
                query_as("INSERT INTO User VALUES (NULL, DEFAULT, ?, ?) RETURNING id")
                    .bind(body.username)
                    .bind(hashed.unwrap())
                    .fetch_one(&data.db)
                    .await
                    .map_err(errors::Errors::Db)?;

            let session: (String, DateTime<Utc>) = query_as(
                "INSERT INTO Session VALUES (?, DEFAULT, DEFAULT, ?) RETURNING id, expires_at",
            )
            .bind(create_id())
            .bind(user.0)
            .fetch_one(&data.db)
            .await
            .map_err(errors::Errors::Db)?;

            return Ok(HttpResponse::Ok()
                .cookie(create_session_cookie(session.0, session.1))
                .finish());
        }
        _ => panic!("{}", format!("Unexpected login kind, {kind}")),
    }
}

#[derive(Deserialize, Debug)]
struct LogoutRequest {
    all: Option<bool>,
}

#[post("/logout")]
async fn logout(
    session: web::ReqData<Session>,
    data: web::Data<AppState>,
    web::Query(query): web::Query<LogoutRequest>,
) -> Result<impl Responder, errors::Errors> {
    if query.all.unwrap_or(false) {
        query!("DELETE FROM Session WHERE user_id = ?", session.user_id)
            .execute(&data.db)
            .await
            .map_err(errors::Errors::Db)?;

        return Ok(remove_tokens_from_response(HttpResponse::Ok().finish()));
    }

    query!("DELETE FROM Session WHERE id = ?", session.id)
        .execute(&data.db)
        .await
        .map_err(errors::Errors::Db)?;

    Ok(remove_tokens_from_response(HttpResponse::Ok().finish()))
}

#[get("/me")]
async fn get_me(
    session: web::ReqData<Session>,
    data: web::Data<AppState>,
) -> Result<impl Responder, errors::Errors> {
    let user = query_as!(
        structures::user::User,
        "SELECT id, created_at, username, password FROM User WHERE id = ?",
        session.user_id
    )
    .fetch_one(&data.db)
    .await
    .map_err(errors::Errors::Db)?;

    Ok(HttpResponse::Ok().json(user))
}
