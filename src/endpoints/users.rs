use std::{collections::HashSet, sync::Mutex};

use actix_web::{web, HttpResponse, Responder};
use cuid2::CuidConstructor;
use once_cell::sync::Lazy;
use password_auth::generate_hash;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{mysql::MySqlDatabaseError, query, Executor, MySql};
use validator::{Validate, ValidationError};

use crate::{
    error::{Error, ErrorResponse},
    middleware::{Identity, Token},
    models::{
        scope::{HasScope, ReadWrite, Scope},
        user::User,
    },
    update_structure,
    ws::{send_updates, WsUpdateEvent},
    AppState,
};

fn validate_is_ascii(s: &str) -> Result<(), ValidationError> {
    if s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        Ok(())
    } else {
        Err(ValidationError::new("username_not_ascii"))
    }
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct RegistrationBody {
    #[serde(deserialize_with = "super::trim_string")]
    #[validate(custom(function = validate_is_ascii), length(min = 3, max = 32))]
    username: String,
    #[serde(default, deserialize_with = "super::trim_opt_string")]
    #[validate(length(min = 2, max = 32))]
    display_name: Option<String>,
    #[validate(length(min = 8, max = 128))]
    password: String,
    #[validate(email, length(max = 255))]
    email: String,
}

static SESSION_ID_GENERATOR: Lazy<CuidConstructor> =
    Lazy::new(|| CuidConstructor::new().with_length(64));

#[derive(Debug, Serialize)]
struct SessionBody {
    token: String,
}

async fn create_session<'a, E: Executor<'a, Database = MySql>>(
    executor: E,
    user_id: u64,
) -> Result<SessionBody, Error> {
    let session_id = SESSION_ID_GENERATOR.create_id();

    query!(
        "INSERT INTO UserSession (id, user_id, created_at, expires_at) VALUES (?, ?, DEFAULT, DEFAULT)",
        session_id,
        user_id
    )
    .execute(executor)
    .await?;

    Ok(SessionBody { token: session_id })
}

pub async fn register_user(
    body: web::Json<RegistrationBody>,
    app_state: web::Data<AppState>,
    generator: web::Data<Mutex<snowflaked::Generator>>,
) -> Result<impl Responder, Error> {
    body.validate()?;

    let mut tx = app_state.db.begin().await?;

    let user_id: u64 = {
        let mut generator = generator.lock().unwrap();
        generator.generate()
    };

    if let Err(err) = query!(
        "INSERT INTO User (id, username, display_name, password, email, email_verified) VALUES (?, ?, ?, ?, ?, FALSE)",
        user_id,
        body.username,
        body.display_name,
        generate_hash(&body.password),
        body.email
    )
    .execute(&mut *tx)
    .await {
        return match err.as_database_error() {
            Some(err)
                if err
                    .try_downcast_ref::<MySqlDatabaseError>()
                    .is_some_and(|err| err.number() == 1062) =>
            {
                Ok(HttpResponse::Conflict().json(ErrorResponse {
                    error: "Username or email already exists".to_string(),
                }))
            }
            _ => Err(err.into()),
        };
    }

    let session = create_session(&mut *tx, user_id).await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(session))
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginBody {
    #[serde(deserialize_with = "super::trim_string")]
    #[validate(custom(function = validate_is_ascii), length(min = 3, max = 32))]
    username: String,
    #[validate(length(min = 8, max = 128))]
    password: String,
}

pub async fn login_user(
    body: web::Json<LoginBody>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, Error> {
    body.validate()?;

    let Some(user_record) = query!(
        "SELECT id, password FROM User WHERE username = ?",
        body.username
    )
    .fetch_optional(&app_state.db)
    .await?
    else {
        return Ok(HttpResponse::Unauthorized().json(ErrorResponse {
            error: "Invalid username or password".to_string(),
        }));
    };

    password_auth::verify_password(&body.password, &user_record.password)?;

    Ok(HttpResponse::Ok().json(create_session(&app_state.db, user_record.id).await?))
}

pub async fn get_user(
    app_state: web::Data<AppState>,
    user_id: web::Path<u64>,
    identity: web::ReqData<Identity>,
) -> Result<impl Responder, Error> {
    if !matches!(identity.into_inner(), Identity::User((_, _))) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let user_id = user_id.into_inner();

    let Some(user) = query!(
        "SELECT username, display_name FROM User WHERE id = ?",
        user_id
    )
    .fetch_optional(&app_state.db)
    .await?
    else {
        return Ok(HttpResponse::NotFound().finish());
    };

    Ok(HttpResponse::Ok().json(User {
        id: user_id,
        username: user.username,
        display_name: user.display_name,
    }))
}

pub async fn get_user_by_username(
    app_state: web::Data<AppState>,
    username: web::Path<String>,
    identity: web::ReqData<Identity>,
) -> Result<impl Responder, Error> {
    if !matches!(identity.into_inner(), Identity::User((_, _))) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let Some(user) = query!(
        "SELECT id, username, display_name FROM User WHERE username = ?",
        username.into_inner()
    )
    .fetch_optional(&app_state.db)
    .await?
    else {
        return Ok(HttpResponse::NotFound().finish());
    };

    Ok(HttpResponse::Ok().json(User {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
    }))
}

pub async fn get_current_user(
    app_state: web::Data<AppState>,
    identity: web::ReqData<Identity>,
) -> Result<impl Responder, Error> {
    let Some(user_id) = identity.has_scope(Scope::Profile(ReadWrite::Read)) else {
        return Ok(HttpResponse::Forbidden().finish());
    };

    let Some(user) = query!(
        "SELECT id, username, display_name, email, email_verified AS `email_verified: bool` FROM User WHERE id = ?",
        user_id
    )
    .fetch_optional(&app_state.db)
    .await? else {
        return Ok(HttpResponse::NotFound().finish());
    };

    let mut value = serde_json::to_value(User {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
    })
    .unwrap();

    value["email"] = Value::String(user.email);
    value["email_verified"] = Value::Bool(user.email_verified);

    Ok(HttpResponse::Ok().json(value))
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserBody {
    #[serde(default, deserialize_with = "super::trim_opt_string")]
    #[validate(custom(function = validate_is_ascii), length(min = 3, max = 32))]
    username: Option<String>,
    #[serde(deserialize_with = "super::deserialize_some_trimmed")]
    #[validate(length(min = 2, max = 32))]
    display_name: Option<Option<String>>,
    #[validate(email, length(max = 255))]
    email: Option<String>,
    #[validate(length(min = 8, max = 128))]
    password: Option<String>,
}

pub async fn update_user(
    app_state: web::Data<AppState>,
    identity: web::ReqData<Identity>,
    body: web::Json<UpdateUserBody>,
) -> Result<impl Responder, Error> {
    body.validate()?;

    let Some(user_id) = identity.has_scope(Scope::Profile(ReadWrite::Write)) else {
        return Ok(HttpResponse::Forbidden().finish());
    };

    let body = body.into_inner();

    let (mut pushed, mut query_builder) =
        update_structure!(raw "User", body, display_name, email, username);

    if body.email.is_some() {
        query_builder.push(", email_verified = FALSE");
    }

    if let Some(password) = body.password {
        if pushed {
            query_builder.push(",");
        }
        pushed = true;
        query_builder
            .push(" password = ")
            .push_bind(generate_hash(password));
    }

    if !pushed {
        return Ok(HttpResponse::BadRequest().finish());
    }

    query_builder
        .push(" WHERE id = ")
        .push_bind(user_id)
        .build()
        .execute(&app_state.db)
        .await?;

    let mut associates = query!(
        r#"
SELECT ServerMember.user_id
FROM ServerMember
INNER JOIN ServerMember AS UpdatedUser ON ServerMember.user_id=?
WHERE ServerMember.server_id=UpdatedUser.server_id

UNION
    
SELECT friend_id AS user_id
FROM UserFriend
WHERE user_id=?

UNION

SELECT user_id
FROM UserFriend
WHERE friend_id=?

UNION

SELECT receiver_id AS user_id
FROM UserFriendRequest
WHERE sender_id=?

UNION

SELECT sender_id AS user_id
FROM UserFriendRequest
WHERE receiver_id=?

UNION

SELECT Other.user_id
FROM DMChannelRecipient
INNER JOIN DMChannelRecipient AS Other ON DMChannelRecipient.channel_id=Other.channel_id
WHERE DMChannelRecipient.user_id=?
"#,
        user_id,
        user_id,
        user_id,
        user_id,
        user_id,
        user_id,
    )
    .fetch_all(&app_state.db)
    .await?
    .into_iter()
    .map(|row| row.user_id)
    .collect::<HashSet<u64>>();

    associates.insert(user_id);

    send_updates(
        [WsUpdateEvent::UserUpdate {
            id: user_id,
            display_name: body.display_name.clone(),
            username: body.username.clone(),
        }],
        &app_state,
        associates,
    );

    Ok(HttpResponse::Ok().finish())
}

pub async fn delete_user(
    app_state: web::Data<AppState>,
    identity: web::ReqData<Identity>,
) -> Result<impl Responder, Error> {
    let user_id = match identity.into_inner() {
        Identity::User((id, None)) => id,
        _ => return Ok(HttpResponse::Forbidden().finish()),
    };

    let mut tx = app_state.db.begin().await?;

    query!(
        "UPDATE User SET began_deletion_at = NOW() WHERE id = ?",
        user_id
    )
    .execute(&mut *tx)
    .await?;

    query!("DELETE FROM UserSession WHERE user_id = ?", user_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Deserialize)]
pub struct LogoutQuery {
    all: Option<bool>,
}

pub async fn logout_user(
    app_state: web::Data<AppState>,
    identity: web::ReqData<Identity>,
    query: web::Query<LogoutQuery>,
    token: web::ReqData<Token>,
) -> Result<impl Responder, Error> {
    let user_id = match identity.into_inner() {
        Identity::User((id, None)) => id,
        _ => return Ok(HttpResponse::Forbidden().finish()),
    };

    if query.all.unwrap_or(false) {
        query!("DELETE FROM UserSession WHERE user_id = ?", user_id)
            .execute(&app_state.db)
            .await?;
    } else {
        query!("DELETE FROM UserSession WHERE id = ?", token.into_inner().0)
            .execute(&app_state.db)
            .await?;
    }

    Ok(HttpResponse::Ok().finish())
}
