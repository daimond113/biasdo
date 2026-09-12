use serde::Serialize;
use ts_rs::TS;

crate::models::id_type!(UserId);

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct User {
	pub id: UserId,
	pub username: String,
	pub display_name: Option<String>,
}
