use crate::models::{channel::Channel, user::User};
use chrono::{DateTime, Utc};
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct UserFriend {
	pub user: User,
	pub friend: User,
	pub created_at: DateTime<Utc>,
	pub channel: Channel,
}
