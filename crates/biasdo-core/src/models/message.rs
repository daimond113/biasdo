use std::{fmt::Display, str::FromStr};

use crate::models::{channel::ChannelId, servermember::ServerMember, user::User};
use serde::Serialize;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, PartialEq, Eq, SerializeDisplay, DeserializeFromStr, TS, Hash)]
pub enum MessageKind {
	#[ts(rename = "text")]
	Text,
}

impl Display for MessageKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			MessageKind::Text => write!(f, "text"),
		}
	}
}

impl FromStr for MessageKind {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"text" => Ok(MessageKind::Text),
			_ => Err(format!("Invalid message kind: {}", s)),
		}
	}
}

crate::models::id_type!(MessageId);

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct Message {
	pub id: MessageId,
	pub kind: MessageKind,
	pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
	pub content: String,
	pub channel_id: ChannelId,
	pub user: User,
	pub member: Option<ServerMember>,
}
