use std::fmt::Display;

use actix_web::{rt, web};
use biasdo_core::models::{
	auth::{has_scope, ReadWrite, Scope},
	channel::{Channel, ChannelId},
	friend::UserFriend,
	friendrequest::UserFriendRequest,
	invite::Invite,
	message::{Message, MessageId},
	server::{Server, ServerId},
	servermember::ServerMember,
	user::UserId,
};
use serde::Serialize;
use ts_rs::TS;

use crate::AppState;

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
		id: ServerId,
		#[serde(skip_serializing_if = "Option::is_none")]
		name: Option<String>,
	},
	ServerDelete {
		id: ServerId,
	},

	ChannelCreate(Channel),
	ChannelUpdate {
		id: ChannelId,
		#[serde(skip_serializing_if = "Option::is_none")]
		name: Option<String>,
	},
	ChannelDelete {
		id: ChannelId,
	},

	MessageCreate(Message),
	MessageUpdate {
		id: MessageId,
		updated_at: chrono::DateTime<chrono::Utc>,
		#[serde(skip_serializing_if = "Option::is_none")]
		content: Option<String>,
	},
	MessageDelete {
		id: MessageId,
	},

	InviteCreate(Invite),
	InviteDelete {
		id: String,
	},

	MemberCreate(ServerMember),
	MemberUpdate {
		user_id: UserId,
		server_id: ServerId,
		#[serde(skip_serializing_if = "Option::is_none")]
		nickname: Option<Option<String>>,
	},
	MemberDelete {
		user_id: UserId,
		server_id: ServerId,
	},

	UserUpdate {
		id: UserId,
		#[serde(skip_serializing_if = "Option::is_none")]
		username: Option<String>,
		#[serde(skip_serializing_if = "Option::is_none")]
		display_name: Option<Option<String>>,
	},

	FriendRequestCreate(UserFriendRequest),
	FriendRequestDelete {
		sender_id: UserId,
		receiver_id: UserId,
	},

	FriendCreate(UserFriend),
	FriendDelete {
		user_id: UserId,
		friend_id: UserId,
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

pub fn send_updates<I: IntoIterator<Item = WsUpdateEvent>, J: IntoIterator<Item = UserId>>(
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
					match scopes {
						Some(scopes) if !has_scope(scopes, *scope) => continue,
						_ => {}
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
