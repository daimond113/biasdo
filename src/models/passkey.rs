use serde::Serialize;
use ts_rs::TS;
use webauthn_rs::prelude::CredentialID;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS, Hash)]
#[ts(export)]
pub struct Passkey {
	#[ts(type = "string")]
	pub id: CredentialID,
	pub display_name: String,
	pub created_at: chrono::DateTime<chrono::Utc>,
}
