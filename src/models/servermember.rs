use crate::models::user::User;
use chrono::{DateTime, Utc};
use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS, Hash)]
#[ts(export)]
pub struct ServerMember {
	#[serde(serialize_with = "super::id_str")]
	#[ts(type = "`${number}`")]
	pub user_id: u64,
	#[serde(serialize_with = "super::id_str")]
	#[ts(type = "`${number}`")]
	pub server_id: u64,
	pub created_at: DateTime<Utc>,
	pub nickname: Option<String>,
	pub user: Option<User>,
}
