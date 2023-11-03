use crate::id_type::Id;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Session {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub user_id: Id,
}
