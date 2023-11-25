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

#[macro_export]
macro_rules! get_member_value {
    ($record:expr) => {{
        let member = structures::member::Member {
            id: $record.member_id.into(),
            created_at: $record.member_created_at,
            server_id: $record.member_server_id.into(),
            user_id: $record.member_user_id.into(),
            nickname: $record.member_nickname.clone(),
        };

        let mut member_value = serde_json::to_value(member.clone()).unwrap();

        if let Some(id) = member.user_id.0 {
            let user = structures::user::User {
                id: id.into(),
                created_at: $record.user_created_at,
                username: $record.user_username.clone(),
                password: "".to_string(),
            };

            merge_json(&mut member_value, &serde_json::json!({ "user": user }));
        }

        member_value
    }};
}

#[macro_export]
macro_rules! get_member_opt_user_value {
    ($record:expr) => {{
        let member = structures::member::Member {
            id: $record.member_id.into(),
            created_at: $record.member_created_at,
            server_id: $record.member_server_id.into(),
            user_id: $record.member_user_id.into(),
            nickname: $record.member_nickname.clone(),
        };

        let mut member_value = serde_json::to_value(member.clone()).unwrap();

        if let Some(id) = member.user_id.0 {
            let user = structures::user::User {
                id: id.into(),
                created_at: $record.user_created_at.unwrap(),
                username: $record.user_username.clone().unwrap(),
                password: "".to_string(),
            };

            merge_json(&mut member_value, &serde_json::json!({ "user": user }));
        }

        member_value
    }};
}
