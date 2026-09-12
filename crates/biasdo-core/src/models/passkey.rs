use serde::Serialize;
use ts_rs::TS;
use webauthn_rs::prelude::CredentialID;

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct Passkey {
	#[ts(type = "string")]
	pub id: CredentialID,
	pub display_name: String,
	pub created_at: chrono::DateTime<chrono::Utc>,
}
