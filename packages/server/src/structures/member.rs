use crate::id_type::{Id, OptionId};
use chrono::{DateTime, Utc};
use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS, Hash)]
#[ts(export, export_to = "../server-utils/src/")]
pub struct Member {
    pub id: Id,
    pub created_at: DateTime<Utc>,
    pub server_id: Id,
    #[serde(skip_serializing_if = "OptionId::is_none")]
    pub user_id: OptionId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
}
