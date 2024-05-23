use std::{fmt::Display, str::FromStr};

use crate::models::user::User;
use serde::Serialize;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, PartialEq, Eq, SerializeDisplay, DeserializeFromStr, TS, Hash)]
pub enum ChannelKind {
    #[ts(rename = "text")]
    Text,
    DM,
}

impl Display for ChannelKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelKind::Text => write!(f, "text"),
            ChannelKind::DM => write!(f, "DM"),
        }
    }
}

impl FromStr for ChannelKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "text" => Ok(ChannelKind::Text),
            "DM" => Ok(ChannelKind::DM),
            _ => Err(format!("Invalid channel kind: {}", s)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS, Hash)]
#[ts(export)]
pub struct Channel {
    #[serde(serialize_with = "super::id_str")]
    #[ts(type = "`${number}`")]
    pub id: u64,
    pub name: String,
    pub kind: ChannelKind,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "super::opt_id_str"
    )]
    #[ts(type = "`${number}`")]
    pub server_id: Option<u64>,
    pub user: Option<User>,
}
