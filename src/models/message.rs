use std::{fmt::Display, str::FromStr};

use crate::models::{servermember::ServerMember, user::User};
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS, Hash)]
#[ts(export)]
pub struct Message {
    #[serde(serialize_with = "super::id_str")]
    #[ts(type = "`${number}`")]
    pub id: u64,
    pub kind: MessageKind,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub content: String,
    #[serde(serialize_with = "super::id_str")]
    #[ts(type = "`${number}`")]
    pub channel_id: u64,
    pub user: User,
    pub member: Option<ServerMember>,
}
