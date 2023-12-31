use crate::id_type::Id;
use chrono::{DateTime, Utc};
use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS, Hash)]
#[ts(export, export_to = "../server-utils/src/")]
pub struct Invite {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub server_id: Id,
}
