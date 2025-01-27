use crate::error::BackendError;
use cuid2::CuidConstructor;
use serde::Serialize;
use sqlx::{query, Executor, MySql};
use std::sync::LazyLock;

#[derive(Debug, Serialize)]
pub struct SessionBody {
	token: String,
}

static SESSION_ID_GENERATOR: LazyLock<CuidConstructor> =
	LazyLock::new(|| CuidConstructor::new().with_length(64));

pub async fn create_session<'a, E: Executor<'a, Database = MySql>>(
	executor: E,
	user_id: u64,
) -> Result<SessionBody, BackendError> {
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
