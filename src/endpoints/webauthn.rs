use crate::{
	error::{ApiResult, BackendError},
	middleware::Identity,
	models::{auth::create_session, id_to_uuid, passkey::Passkey as PasskeyResponse, uuid_to_id},
	update_structure, AppState,
};
use actix_web::{
	cookie::{time::OffsetDateTime, Cookie, SameSite},
	web, HttpRequest, HttpResponse,
};
use cuid2::CuidConstructor;
use sqlx::{query, MySqlConnection, Row};
use std::sync::LazyLock;
use validator::Validate;
use webauthn_rs::{prelude::*, DEFAULT_AUTHENTICATOR_TIMEOUT};

static WEBAUTHN_ID_GENERATOR: LazyLock<CuidConstructor> =
	LazyLock::new(|| CuidConstructor::new().with_length(32));

const REGISTRATION_ID_COOKIE_NAME: &str = "biasdo-passreg";
const AUTHENTICATION_ID_COOKIE_NAME: &str = "biasdo-passauth";

fn make_cookie<'a>(name: &'a str, value: &'a str) -> Cookie<'a> {
	Cookie::build(name, value)
		.http_only(true)
		.secure(true)
		.max_age(DEFAULT_AUTHENTICATOR_TIMEOUT.try_into().unwrap())
		.expires(OffsetDateTime::now_utc() + DEFAULT_AUTHENTICATOR_TIMEOUT)
		.same_site(SameSite::None)
		.finish()
}

async fn handle_auth_res(
	res: AuthenticationResult,
	tx: &mut MySqlConnection,
) -> Result<(), BackendError> {
	if !res.needs_update() {
		return Ok(());
	}

	let mut sk = query!(
		r#"SELECT cred AS `cred: sqlx::types::Json<Passkey>`
FROM WebauthnUserCredential
WHERE cred_id=?"#,
		res.cred_id().as_slice()
	)
	.fetch_one(&mut *tx)
	.await?
	.cred
	.0;

	if sk.update_credential(&res).is_some_and(|u| u) {
		query!(
			r#"UPDATE WebauthnUserCredential
SET cred=?
WHERE cred_id=?"#,
			serde_json::to_string(&sk)?,
			res.cred_id().as_slice()
		)
		.execute(&mut *tx)
		.await?;
	};

	Ok(())
}

pub async fn start_register_passkey(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
) -> ApiResult {
	let Identity::User(id) = identity.into_inner() else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let records = query!(
		r#"SELECT User.username, User.display_name, WebauthnUserCredential.cred_id
FROM User
LEFT JOIN WebauthnUserCredential ON WebauthnUserCredential.user_id=User.id
WHERE User.id = ?"#,
		id
	)
	.fetch_all(&app_state.db)
	.await?;

	let (username, display_name) = {
		let user = records.first().unwrap();

		(user.username.clone(), user.display_name.clone())
	};

	let (ccr, reg_state) = app_state.webauthn.start_passkey_registration(
		id_to_uuid(id),
		&username,
		display_name.as_deref().unwrap_or(&username),
		Some(
			records
				.into_iter()
				.filter_map(|r| r.cred_id)
				.map(CredentialID::from)
				.collect::<Vec<_>>(),
		)
		.filter(|v| !v.is_empty()),
	)?;

	let reg_id = WEBAUTHN_ID_GENERATOR.create_id();

	query!(
		"INSERT INTO WebauthnPasskeyRegistration (user_id, reg_id, reg_state, expires_at) VALUES (?, ?, ?, DEFAULT)",
		id,
		reg_id,
		serde_json::to_string(&reg_state)?
	)
		.execute(&app_state.db)
		.await?;

	Ok(HttpResponse::Ok()
		.cookie(make_cookie(REGISTRATION_ID_COOKIE_NAME, &reg_id))
		.json(ccr))
}

pub async fn finish_register_passkey(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	request: HttpRequest,
	reg: web::Json<RegisterPublicKeyCredential>,
) -> ApiResult {
	let Identity::User(id) = identity.into_inner() else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let Some(mut cookie) = request.cookie("biasdo-passreg") else {
		return Ok(HttpResponse::Unauthorized().finish());
	};

	let mut tx = app_state.db.begin().await?;

	let Some(row) = query!(
		r#"DELETE
FROM WebauthnPasskeyRegistration
WHERE user_id=? AND reg_id=? AND expires_at > NOW()
RETURNING reg_state"#,
		id,
		cookie.value()
	)
	.fetch_optional(&mut *tx)
	.await?
	else {
		return Ok(HttpResponse::Unauthorized().finish());
	};

	let reg_state = serde_json::from_slice(row.get(0))?;

	let sk = app_state
		.webauthn
		.finish_passkey_registration(&reg, &reg_state)?;

	let created_at = chrono::Utc::now();

	match query!(
		r#"INSERT INTO WebauthnUserCredential (user_id, cred_id, cred, display_name, created_at) VALUES (?, ?, ?, 'Passkey', ?)"#,
		id,
		sk.cred_id().as_slice(),
		serde_json::to_string(&sk)?,
		created_at
	)
	.execute(&mut *tx)
	.await
	{
		Err(e)
			if e.as_database_error()
				.is_some_and(|e| e.is_unique_violation()) =>
		{
			return Ok(HttpResponse::Conflict().finish());
		}
		r => r?,
	};

	tx.commit().await?;

	cookie.make_removal();
	Ok(HttpResponse::Ok().cookie(cookie).json(PasskeyResponse {
		id: sk.cred_id().clone(),
		display_name: "Passkey".to_string(),
		created_at,
	}))
}

#[derive(Debug, serde::Deserialize)]
pub struct AuthenticationStartBody {
	username: String,
}

pub async fn start_authentication(
	body: web::Json<AuthenticationStartBody>,
	app_state: web::Data<AppState>,
) -> ApiResult {
	let results = query!(
		r#"SELECT User.id, WebauthnUserCredential.cred AS `cred: sqlx::types::Json<Passkey>`
FROM User
INNER JOIN WebauthnUserCredential ON WebauthnUserCredential.user_id=User.id
WHERE User.username=?"#,
		body.username
	)
	.fetch_all(&app_state.db)
	.await?;

	let user_id = {
		let Some(row) = results.first() else {
			return Ok(HttpResponse::NotFound().finish());
		};

		row.id
	};

	let (rcr, auth_state) = app_state
		.webauthn
		.start_passkey_authentication(&results.into_iter().map(|r| r.cred.0).collect::<Vec<_>>())?;

	let auth_id = WEBAUTHN_ID_GENERATOR.create_id();

	query!(
		"INSERT INTO WebauthnAuthState (user_id, auth_id, state, expires_at) VALUES (?, ?, ?, DEFAULT)",
		user_id,
		auth_id,
		serde_json::to_string(&auth_state)?
	)
	.execute(&app_state.db)
	.await?;

	Ok(HttpResponse::Ok()
		.cookie(make_cookie(AUTHENTICATION_ID_COOKIE_NAME, &auth_id))
		.json(rcr))
}

pub async fn finish_authentication(
	app_state: web::Data<AppState>,
	request: HttpRequest,
	auth: web::Json<PublicKeyCredential>,
) -> ApiResult {
	let Some(mut cookie) = request.cookie("biasdo-passauth") else {
		return Ok(HttpResponse::Unauthorized().finish());
	};

	let mut tx = app_state.db.begin().await?;

	let Some(row) = query!(
		r#"DELETE
FROM WebauthnAuthState
WHERE auth_id=? AND expires_at > NOW()
RETURNING user_id, state"#,
		cookie.value()
	)
	.fetch_optional(&mut *tx)
	.await?
	else {
		return Ok(HttpResponse::Unauthorized().finish());
	};

	let (user_id, state): (u64, _) = (row.get(0), serde_json::from_slice(row.get(1))?);

	let res = app_state
		.webauthn
		.finish_passkey_authentication(&auth, &state)?;

	handle_auth_res(res, &mut tx).await?;

	let session = create_session(&mut *tx, user_id).await?;
	tx.commit().await?;

	cookie.make_removal();
	Ok(HttpResponse::Ok().cookie(cookie).json(session))
}

pub async fn start_conditional_authentication(app_state: web::Data<AppState>) -> ApiResult {
	let (rcr, cond_state) = app_state.webauthn.start_discoverable_authentication()?;

	let cond_id = WEBAUTHN_ID_GENERATOR.create_id();

	query!(
		"INSERT INTO WebauthnConditionalAuthState (cond_id, state, expires_at) VALUES (?, ?, DEFAULT)",
		cond_id,
		serde_json::to_string(&cond_state)?
	)
	.execute(&app_state.db)
	.await?;

	Ok(HttpResponse::Ok()
		.cookie(make_cookie(AUTHENTICATION_ID_COOKIE_NAME, &cond_id))
		.json(rcr))
}

pub async fn finish_conditional_authentication(
	app_state: web::Data<AppState>,
	request: HttpRequest,
	auth: web::Json<PublicKeyCredential>,
) -> ApiResult {
	let Some(mut cookie) = request.cookie(AUTHENTICATION_ID_COOKIE_NAME) else {
		return Ok(HttpResponse::Unauthorized().finish());
	};

	let mut tx = app_state.db.begin().await?;

	let Some(row) = query!(
		r#"DELETE
FROM WebauthnConditionalAuthState
WHERE cond_id=? AND expires_at > NOW()
RETURNING state"#,
		cookie.value()
	)
	.fetch_optional(&mut *tx)
	.await?
	else {
		return Ok(HttpResponse::Unauthorized().finish());
	};

	let state = serde_json::from_slice(row.get(0))?;

	let (id, _) = app_state
		.webauthn
		.identify_discoverable_authentication(&auth)?;
	let id = uuid_to_id(id);

	let creds = query!(
		r#"SELECT cred AS `cred: sqlx::types::Json<Passkey>`
FROM WebauthnUserCredential
WHERE user_id=?"#,
		id
	)
	.fetch_all(&mut *tx)
	.await?;

	let res = app_state.webauthn.finish_discoverable_authentication(
		&auth,
		state,
		&creds
			.into_iter()
			.map(|r| r.cred.0.into())
			.collect::<Vec<_>>(),
	)?;

	handle_auth_res(res, &mut tx).await?;

	let session = create_session(&mut *tx, id).await?;
	tx.commit().await?;

	cookie.make_removal();
	Ok(HttpResponse::Ok().cookie(cookie).json(session))
}

pub async fn get_user_passkeys(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
) -> ApiResult {
	let Identity::User(id) = identity.into_inner() else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let passkeys = query!(
		r#"SELECT cred_id, display_name, created_at
FROM WebauthnUserCredential
WHERE user_id=?
ORDER BY created_at ASC"#,
		id
	)
	.fetch_all(&app_state.db)
	.await?;

	Ok(HttpResponse::Ok().json(
		passkeys
			.into_iter()
			.map(|row| PasskeyResponse {
				id: row.cred_id.into(),
				display_name: row.display_name,
				created_at: row.created_at,
			})
			.collect::<Vec<_>>(),
	))
}

pub async fn get_user_passkey(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	request: HttpRequest,
) -> ApiResult {
	let cred_id = request.match_info().get("cred_id").unwrap();
	let deser = serde::de::value::StrDeserializer::<serde::de::value::Error>::new(cred_id);
	let cred_id: CredentialID = serde::Deserialize::deserialize(deser)?;

	let Identity::User(id) = identity.into_inner() else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let Some(passkey) = query!(
		r#"SELECT display_name, created_at
FROM WebauthnUserCredential
WHERE user_id=? AND cred_id=?"#,
		id,
		cred_id.as_slice()
	)
	.fetch_optional(&app_state.db)
	.await?
	else {
		return Ok(HttpResponse::NotFound().finish());
	};

	Ok(HttpResponse::Ok().json(PasskeyResponse {
		id: cred_id,
		display_name: passkey.display_name,
		created_at: passkey.created_at,
	}))
}

#[derive(Debug, serde::Deserialize, Validate)]
pub struct UpdatePasskeyBody {
	#[serde(default, deserialize_with = "super::trim_opt_string")]
	#[validate(length(min = 1, max = 64))]
	display_name: Option<String>,
}

pub async fn update_user_passkey(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	request: HttpRequest,
	body: web::Json<UpdatePasskeyBody>,
) -> ApiResult {
	body.validate()?;

	let cred_id = request.match_info().get("cred_id").unwrap();
	let deser = serde::de::value::StrDeserializer::<serde::de::value::Error>::new(cred_id);
	let cred_id: CredentialID = serde::Deserialize::deserialize(deser)?;

	let Identity::User(id) = identity.into_inner() else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let result = update_structure!("WebauthnUserCredential", body, display_name)
		.push(" WHERE user_id = ")
		.push_bind(id)
		.push(" AND cred_id = ")
		.push_bind(cred_id.as_slice())
		.build()
		.execute(&app_state.db)
		.await?;

	if result.rows_affected() == 0 {
		return Ok(HttpResponse::NotFound().finish());
	}

	Ok(HttpResponse::Ok().finish())
}

pub async fn delete_user_passkey(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	request: HttpRequest,
) -> ApiResult {
	let cred_id = request.match_info().get("cred_id").unwrap();
	let deser = serde::de::value::StrDeserializer::<serde::de::value::Error>::new(cred_id);
	let cred_id: CredentialID = serde::Deserialize::deserialize(deser)?;

	let Identity::User(id) = identity.into_inner() else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let result = query!(
		r#"DELETE
FROM WebauthnUserCredential
WHERE user_id=? AND cred_id=?"#,
		id,
		cred_id.as_slice()
	)
	.execute(&app_state.db)
	.await?;

	if result.rows_affected() == 0 {
		return Ok(HttpResponse::NotFound().finish());
	}

	Ok(HttpResponse::Ok().finish())
}
