use serde::Serializer;

pub mod channel;
pub mod client;
pub mod friend;
pub mod friendrequest;
pub mod invite;
pub mod message;
pub mod scope;
pub mod server;
pub mod servermember;
pub mod user;

// sending 64-bit integers will not work in JavaScript and other languages
pub fn id_str<S: Serializer>(id: &u64, s: S) -> Result<S::Ok, S::Error> {
	s.serialize_str(&id.to_string())
}

pub fn opt_id_str<S: Serializer>(id: &Option<u64>, s: S) -> Result<S::Ok, S::Error> {
	match id {
		Some(id) => s.serialize_str(&id.to_string()),
		None => s.serialize_none(),
	}
}
