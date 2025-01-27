use crate::{error::ApiResult, middleware::Identity, models::id_to_uuid, AppState};
use actix_web::{cookie::Cookie, web, HttpRequest, HttpResponse};
use cuid2::CuidConstructor;
use sqlx::{query, Row};
use std::sync::LazyLock;
use webauthn_rs::prelude::{CredentialID, RegisterPublicKeyCredential};

static REGISTRATION_ID_GENERATOR: LazyLock<CuidConstructor> =
	LazyLock::new(|| CuidConstructor::new().with_length(32));

const REGISTRATION_ID_COOKIE_NAME: &str = "biasdo-passreg";

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

	let reg_id = REGISTRATION_ID_GENERATOR.create_id();

	query!(
		"INSERT INTO WebauthnPasskeyRegistration (user_id, reg_id, reg_state) VALUES (?, ?, ?)",
		id,
		reg_id,
		serde_json::to_string(&reg_state)?
	)
	.execute(&app_state.db)
	.await?;

	Ok(HttpResponse::Ok()
		.cookie(
			Cookie::build(REGISTRATION_ID_COOKIE_NAME, reg_id)
				.http_only(true)
				.secure(true)
				.finish(),
		)
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
WHERE user_id=? AND reg_id=?
RETURNING reg_state"#,
		id,
		cookie.value()
	)
	.fetch_optional(&mut *tx)
	.await?
	else {
		return Ok(HttpResponse::Unauthorized().finish());
	};

	let reg_state = serde_json::from_str(row.get("reg_state"))?;

	let sk = app_state
		.webauthn
		.finish_passkey_registration(&reg, &reg_state)?;

	match query!(
		r#"INSERT INTO WebauthnUserCredential (user_id, cred_id, cred) VALUES (?, ?, ?)"#,
		id,
		sk.cred_id().as_slice(),
		serde_json::to_string(&sk)?
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
	Ok(HttpResponse::Ok().cookie(cookie).finish())
}
