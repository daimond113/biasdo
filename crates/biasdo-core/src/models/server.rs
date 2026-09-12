use serde::Serialize;
use ts_rs::TS;

use crate::models::user::UserId;

crate::models::id_type!(ServerId);

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct Server {
	pub id: ServerId,
	pub name: String,
	pub owner_id: UserId,
}
