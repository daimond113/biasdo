use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS, Hash)]
#[ts(export)]
pub struct User {
	#[serde(serialize_with = "super::id_str")]
	#[ts(type = "`${number}`")]
	pub id: u64,
	pub username: String,
	pub display_name: Option<String>,
}
