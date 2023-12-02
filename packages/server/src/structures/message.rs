use std::str::FromStr;

use crate::id_type::{Id, OptionId};
use chrono::{DateTime, Utc};
use derive_more::Display;
use serde::Serialize;
use sqlx::Type;
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Display, Type, TS, Hash)]
#[ts(export, export_to = "../server-utils/src/")]
pub enum MessageKind {
    Text,
}

impl FromStr for MessageKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Text" => Ok(MessageKind::Text),
            _ => Err(format!("Unknown message kind: {}", s)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS, Hash)]
#[ts(export, export_to = "../server-utils/src/")]
pub struct Message {
    pub id: Id,
    pub created_at: DateTime<Utc>,
    pub content: String,
    pub kind: MessageKind,
    pub channel_id: Id,
    #[serde(skip_serializing_if = "OptionId::is_none")]
    pub user_id: OptionId,
}
