use serde::Serialize;
use ts_rs::TS;
use url::Url;

use crate::models::user::UserId;

crate::models::id_type!(ClientId);

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct Client {
	pub id: ClientId,
	pub name: String,
	pub owner_id: UserId,
	pub client_uri: Option<Url>,
	pub tos_uri: Option<Url>,
	pub policy_uri: Option<Url>,
	pub redirect_uris: Vec<Url>,
}
