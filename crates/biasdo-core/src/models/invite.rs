use crate::models::server::Server;
use chrono::{DateTime, Utc};
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct Invite {
	pub id: String,
	pub server: Server,
	pub created_at: DateTime<Utc>,
	pub expires_at: DateTime<Utc>,
}
