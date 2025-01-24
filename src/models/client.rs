use serde::Serialize;
use ts_rs::TS;
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS, Hash)]
#[ts(export)]
pub struct Client {
	#[serde(serialize_with = "super::id_str")]
	#[ts(type = "`${number}`")]
	pub id: u64,
	pub name: String,
	#[serde(serialize_with = "super::id_str")]
	#[ts(type = "`${number}`")]
	pub owner_id: u64,
	pub client_uri: Option<Url>,
	pub tos_uri: Option<Url>,
	pub policy_uri: Option<Url>,
	pub redirect_uris: Vec<Url>,
}
