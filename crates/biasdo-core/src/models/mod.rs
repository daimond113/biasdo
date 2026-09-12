pub mod auth;
pub use auth::*;
pub mod channel;
pub use channel::*;
pub mod client;
pub use client::*;
pub mod friend;
pub use friend::*;
pub mod friendrequest;
pub use friendrequest::*;
pub mod invite;
pub use invite::*;
pub mod message;
pub use message::*;
pub mod passkey;
pub use passkey::*;
pub mod server;
pub use server::*;
pub mod servermember;
pub use servermember::*;
pub mod user;
pub use user::*;

macro_rules! id_type {
	($name:ident) => {
		#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type, ts_rs::TS)]
		#[ts(type = "`${number}`")]
		pub struct $name(u64);

		impl std::fmt::Display for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				write!(f, "{}", self.0)
			}
		}

		impl serde::Serialize for $name {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
			where
				S: serde::Serializer,
			{
				serializer.collect_str(self)
			}
		}

		impl<'de> serde::Deserialize<'de> for $name {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: serde::Deserializer<'de>,
			{
				let string = String::deserialize(deserializer)?;
				Ok(Self(string.parse().map_err(serde::de::Error::custom)?))
			}
		}
	};
}
pub(crate) use id_type;

pub fn id_to_uuid(id: u64) -> webauthn_rs::prelude::Uuid {
	webauthn_rs::prelude::Uuid::from_u64_pair(0, id)
}

pub fn uuid_to_id(uuid: webauthn_rs::prelude::Uuid) -> u64 {
	uuid.as_u64_pair().1
}
