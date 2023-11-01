use chrono::{DateTime, Utc};
use serde::Serialize;
use ts_rs::TS;
use crate::id_type::Id;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS, Hash)]
#[ts(export, export_to = "../server-utils/src/")]
pub struct Server {
    pub id: Id,
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub owner_id: Id,
}