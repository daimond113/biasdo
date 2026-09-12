use crate::models::user::User;
use chrono::{DateTime, Utc};
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct UserFriendRequest {
	pub sender: User,
	pub receiver: User,
	pub created_at: DateTime<Utc>,
}
