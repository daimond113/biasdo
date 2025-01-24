use crate::models::user::User;
use chrono::{DateTime, Utc};
use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS, Hash)]
#[ts(export)]
pub struct UserFriendRequest {
	pub sender: User,
	pub receiver: User,
	pub created_at: DateTime<Utc>,
}
