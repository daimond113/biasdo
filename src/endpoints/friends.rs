use crate::{
	error::BackendError,
	middleware::Identity,
	models::{
		channel::{Channel, ChannelKind},
		friend::UserFriend,
		scope::{HasScope, ReadWrite, Scope},
		user::User,
	},
	ws::{send_updates, WsUpdateEvent},
	AppState,
};
use actix_web::{web, HttpResponse, Responder};
use sqlx::query;

macro_rules! user_friend_row {
	($row:expr) => {{
		let user = User {
			id: $row.user_id,
			username: $row.user_username,
			display_name: $row.user_display_name,
		};

		UserFriend {
			friend: User {
				id: $row.friend_id,
				username: $row.friend_username,
				display_name: $row.friend_display_name,
			},
			user: user.clone(),
			channel: Channel {
				id: $row.channel_id,
				name: "".to_string(),
				kind: ChannelKind::DM,
				server_id: None,
				user: Some(user),
			},
			created_at: $row.created_at,
		}
	}};
}

pub async fn get_friends(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
) -> Result<impl Responder, BackendError> {
	let Some(user_id) = identity.has_scope(Scope::Friends(ReadWrite::Read)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let friends = query!(
		r#"SELECT UserFriend.friend_id, UserFriend.user_id, UserFriend.created_at,
Friend.username AS `friend_username`, Friend.display_name AS `friend_display_name`,
User.username AS `user_username`, User.display_name AS `user_display_name`,
Channel.id AS `channel_id`
FROM UserFriend
INNER JOIN User AS Friend ON UserFriend.friend_id=Friend.id
INNER JOIN User ON UserFriend.user_id=User.id
INNER JOIN DMChannelRecipient AS UserRecipient ON UserRecipient.user_id=User.id
INNER JOIN DMChannelRecipient AS FriendRecipient ON FriendRecipient.user_id=Friend.id
INNER JOIN Channel ON UserRecipient.channel_id=Channel.id AND FriendRecipient.channel_id=Channel.id
WHERE ? IN (UserFriend.user_id, UserFriend.friend_id) AND Channel.kind = 'DM'
"#,
		user_id
	)
	.fetch_all(&app_state.db)
	.await?;

	Ok(HttpResponse::Ok().json(
		friends
			.into_iter()
			.map(|row| user_friend_row!(row))
			.collect::<Vec<_>>(),
	))
}

pub async fn get_friend(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	path: web::Path<u64>,
) -> Result<impl Responder, BackendError> {
	let Some(user_id) = identity.has_scope(Scope::Friends(ReadWrite::Read)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let friend_id = path.into_inner();

	let Some(friend) = query!(
        r#"SELECT UserFriend.friend_id, UserFriend.user_id, UserFriend.created_at,
Friend.username AS `friend_username`, Friend.display_name AS `friend_display_name`,
User.username AS `user_username`, User.display_name AS `user_display_name`,
Channel.id AS `channel_id`
FROM UserFriend
INNER JOIN User AS Friend ON UserFriend.friend_id=Friend.id
INNER JOIN User ON UserFriend.user_id=User.id
INNER JOIN DMChannelRecipient AS UserRecipient ON UserRecipient.user_id=User.id
INNER JOIN DMChannelRecipient AS FriendRecipient ON FriendRecipient.user_id=Friend.id
INNER JOIN Channel ON UserRecipient.channel_id=Channel.id AND FriendRecipient.channel_id=Channel.id
WHERE (UserFriend.user_id, UserFriend.friend_id) = (?, ?) OR (UserFriend.user_id, UserFriend.friend_id) = (?, ?) AND Channel.kind = 'DM'
"#,
        user_id,
        friend_id,
        friend_id,
        user_id
    )
    .fetch_optional(&app_state.db)
    .await? else {
        return Ok(HttpResponse::NotFound().finish());
    };

	Ok(HttpResponse::Ok().json(user_friend_row!(friend)))
}

pub async fn delete_friend(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	path: web::Path<u64>,
) -> Result<impl Responder, BackendError> {
	let Some(user_id) = identity.has_scope(Scope::Friends(ReadWrite::Write)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let friend_id = path.into_inner();

	let result = query!(
        "DELETE FROM UserFriend WHERE (user_id, friend_id) = (?, ?) OR (user_id, friend_id) = (?, ?)",
        user_id,
        friend_id,
        friend_id,
        user_id
    )
    .execute(&app_state.db)
    .await?;

	if result.rows_affected() == 0 {
		return Ok(HttpResponse::NotFound().finish());
	}

	send_updates(
		[WsUpdateEvent::FriendDelete { user_id, friend_id }],
		&app_state,
		[user_id, friend_id],
	);

	Ok(HttpResponse::Ok().finish())
}
