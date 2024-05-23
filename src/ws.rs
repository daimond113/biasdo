use std::fmt::Display;

use actix_web::{rt, web};
use serde::Serialize;
use ts_rs::TS;

use crate::{
    models::{
        channel::Channel,
        friend::UserFriend,
        friendrequest::UserFriendRequest,
        invite::Invite,
        message::Message,
        scope::{HasScope, ReadWrite, Scope},
        server::Server,
        servermember::ServerMember,
    },
    AppState,
};

#[derive(Debug, Serialize, TS)]
#[ts(export)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum WsUpdateEvent {
    // only here for TypeScript type generation
    #[cfg(test)]
    #[allow(dead_code)]
    Reauthenticate,

    ServerCreate(Server),
    ServerUpdate {
        #[serde(serialize_with = "crate::models::id_str")]
        #[ts(type = "`${number}`")]
        id: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    ServerDelete {
        #[serde(serialize_with = "crate::models::id_str")]
        #[ts(type = "`${number}`")]
        id: u64,
    },

    ChannelCreate(Channel),
    ChannelUpdate {
        #[serde(serialize_with = "crate::models::id_str")]
        #[ts(type = "`${number}`")]
        id: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    ChannelDelete {
        #[serde(serialize_with = "crate::models::id_str")]
        #[ts(type = "`${number}`")]
        id: u64,
    },

    MessageCreate(Message),
    MessageUpdate {
        #[serde(serialize_with = "crate::models::id_str")]
        #[ts(type = "`${number}`")]
        id: u64,
        updated_at: chrono::DateTime<chrono::Utc>,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
    },
    MessageDelete {
        #[serde(serialize_with = "crate::models::id_str")]
        #[ts(type = "`${number}`")]
        id: u64,
    },

    InviteCreate(Invite),
    InviteDelete {
        id: String,
    },

    MemberCreate(ServerMember),
    MemberUpdate {
        #[serde(serialize_with = "crate::models::id_str")]
        #[ts(type = "`${number}`")]
        user_id: u64,
        #[serde(serialize_with = "crate::models::id_str")]
        #[ts(type = "`${number}`")]
        server_id: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        nickname: Option<Option<String>>,
    },
    MemberDelete {
        #[serde(serialize_with = "crate::models::id_str")]
        #[ts(type = "`${number}`")]
        user_id: u64,
        #[serde(serialize_with = "crate::models::id_str")]
        #[ts(type = "`${number}`")]
        server_id: u64,
    },

    UserUpdate {
        #[serde(serialize_with = "crate::models::id_str")]
        #[ts(type = "`${number}`")]
        id: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        username: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        display_name: Option<Option<String>>,
    },

    FriendRequestCreate(UserFriendRequest),
    FriendRequestDelete {
        #[serde(serialize_with = "crate::models::id_str")]
        #[ts(type = "`${number}`")]
        sender_id: u64,
        #[serde(serialize_with = "crate::models::id_str")]
        #[ts(type = "`${number}`")]
        receiver_id: u64,
    },

    FriendCreate(UserFriend),
    FriendDelete {
        #[serde(serialize_with = "crate::models::id_str")]
        #[ts(type = "`${number}`")]
        user_id: u64,
        #[serde(serialize_with = "crate::models::id_str")]
        #[ts(type = "`${number}`")]
        friend_id: u64,
    },
}

impl Display for WsUpdateEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl WsUpdateEvent {
    fn scope_for(&self) -> Scope {
        match self {
            // doesn't matter
            #[cfg(test)]
            WsUpdateEvent::Reauthenticate => Scope::Profile(ReadWrite::Read),

            WsUpdateEvent::ServerCreate { .. } => Scope::Servers(ReadWrite::Read),
            WsUpdateEvent::ServerUpdate { .. } => Scope::Servers(ReadWrite::Read),
            WsUpdateEvent::ServerDelete { .. } => Scope::Servers(ReadWrite::Read),

            WsUpdateEvent::ChannelCreate { .. } => Scope::Servers(ReadWrite::Read),
            WsUpdateEvent::ChannelUpdate { .. } => Scope::Servers(ReadWrite::Read),
            WsUpdateEvent::ChannelDelete { .. } => Scope::Servers(ReadWrite::Read),

            WsUpdateEvent::MessageCreate { .. } => Scope::Messages(ReadWrite::Read),
            WsUpdateEvent::MessageUpdate { .. } => Scope::Messages(ReadWrite::Read),
            WsUpdateEvent::MessageDelete { .. } => Scope::Messages(ReadWrite::Read),

            WsUpdateEvent::InviteCreate { .. } => Scope::Servers(ReadWrite::Read),
            WsUpdateEvent::InviteDelete { .. } => Scope::Servers(ReadWrite::Read),

            WsUpdateEvent::MemberCreate { .. } => Scope::Servers(ReadWrite::Read),
            WsUpdateEvent::MemberUpdate { .. } => Scope::Servers(ReadWrite::Read),
            WsUpdateEvent::MemberDelete { .. } => Scope::Servers(ReadWrite::Read),

            WsUpdateEvent::UserUpdate { .. } => Scope::Profile(ReadWrite::Read),

            WsUpdateEvent::FriendRequestCreate { .. } => Scope::Friends(ReadWrite::Read),
            WsUpdateEvent::FriendRequestDelete { .. } => Scope::Friends(ReadWrite::Read),

            WsUpdateEvent::FriendCreate { .. } => Scope::Friends(ReadWrite::Read),
            WsUpdateEvent::FriendDelete { .. } => Scope::Friends(ReadWrite::Read),
        }
    }
}

pub fn send_updates<I: IntoIterator<Item = WsUpdateEvent>, J: IntoIterator<Item = u64>>(
    events: I,
    app_state: &web::Data<AppState>,
    users: J,
) {
    let events = events
        .into_iter()
        .map(|event| (event.scope_for(), event.to_string()))
        .collect::<Vec<_>>();

    for user_id in users {
        if let Some(rf) = app_state.user_connections.get(&user_id) {
            for (scopes, session) in rf.values() {
                for (scope, json) in &events {
                    if scopes.has_scope(*scope).is_none() {
                        continue;
                    }

                    let mut session = session.clone();
                    let json = json.to_string();

                    rt::spawn(async move {
                        let _ = session.text(json).await;
                    });
                }
            }
        }
    }
}
