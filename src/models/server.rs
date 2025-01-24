use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS, Hash)]
#[ts(export)]
pub struct Server {
	#[serde(serialize_with = "super::id_str")]
	#[ts(type = "`${number}`")]
	pub id: u64,
	pub name: String,
	#[serde(serialize_with = "super::id_str")]
	#[ts(type = "`${number}`")]
	pub owner_id: u64,
}
