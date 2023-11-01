use std::str::FromStr;

use crate::id_type::Id;
use chrono::{DateTime, Utc};
use derive_more::Display;
use serde::Serialize;
use sqlx::Type;
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Display, Type, TS, Hash)]
#[ts(export, export_to = "../server-utils/src/")]
pub enum ChannelKind {
    Text,
}

impl FromStr for ChannelKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Text" => Ok(ChannelKind::Text),
            _ => Err(format!("Unknown channel kind: {}", s)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS, Hash)]
#[ts(export, export_to = "../server-utils/src/")]
pub struct Channel {
    pub id: Id,
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub kind: ChannelKind,
    pub server_id: Id,
}
