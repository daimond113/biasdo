use crate::models::{
	server::ServerId,
	user::{User, UserId},
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ServerMember {
	pub user_id: UserId,
	pub server_id: ServerId,
	pub created_at: DateTime<Utc>,
	pub nickname: Option<String>,
	pub user: Option<User>,
}
