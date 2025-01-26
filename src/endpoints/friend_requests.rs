use actix_web::{web, HttpResponse, Responder};
use sqlx::{mysql::MySqlDatabaseError, query};
use std::sync::Mutex;

use crate::{
	error::{BackendError, ErrorResponse},
	middleware::Identity,
	models::{
		channel::{Channel, ChannelKind},
		friend::UserFriend,
		friendrequest::UserFriendRequest,
		scope::{HasScope, ReadWrite, Scope},
		user::User,
	},
	ws::{send_updates, WsUpdateEvent},
	AppState,
};

macro_rules! user_friend_request_row {
	($row:expr) => {
		UserFriendRequest {
			sender: User {
				id: $row.sender_id,
				username: $row.sender_username,
				display_name: $row.sender_display_name,
			},
			receiver: User {
				id: $row.receiver_id,
				username: $row.receiver_username,
				display_name: $row.receiver_display_name,
			},
			created_at: $row.created_at,
		}
	};
}

pub async fn get_friend_requests(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
) -> Result<impl Responder, BackendError> {
	let Some(user_id) = identity.has_scope(Scope::Friends(ReadWrite::Read)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let requests = query!(
        r#"SELECT UserFriendRequest.sender_id, UserFriendRequest.receiver_id, UserFriendRequest.created_at,
Sender.username AS `sender_username`, Sender.display_name AS `sender_display_name`,
Receiver.username AS `receiver_username`, Receiver.display_name AS `receiver_display_name`
FROM UserFriendRequest
INNER JOIN User AS Sender ON UserFriendRequest.sender_id=Sender.id
INNER JOIN User AS Receiver ON UserFriendRequest.receiver_id=Receiver.id
WHERE ? IN (UserFriendRequest.sender_id, UserFriendRequest.receiver_id)
"#,
        user_id
    )
    .fetch_all(&app_state.db)
    .await?;

	Ok(HttpResponse::Ok().json(
		requests
			.into_iter()
			.map(|row| user_friend_request_row!(row))
			.collect::<Vec<_>>(),
	))
}

pub async fn get_friend_request(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	path: web::Path<u64>,
) -> Result<impl Responder, BackendError> {
	let Some(user_id) = identity.has_scope(Scope::Friends(ReadWrite::Read)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let target_id = path.into_inner();

	let Some(row) = query!(
        r#"SELECT UserFriendRequest.sender_id, UserFriendRequest.receiver_id, UserFriendRequest.created_at,
Sender.username AS `sender_username`, Sender.display_name AS `sender_display_name`,
Receiver.username AS `receiver_username`, Receiver.display_name AS `receiver_display_name`
FROM UserFriendRequest
INNER JOIN User AS Sender ON UserFriendRequest.sender_id=Sender.id
INNER JOIN User AS Receiver ON UserFriendRequest.receiver_id=Receiver.id
WHERE (UserFriendRequest.sender_id, UserFriendRequest.receiver_id) = (?, ?) OR (UserFriendRequest.sender_id, UserFriendRequest.receiver_id) = (?, ?)
"#,
        user_id,
        target_id,
        target_id,
        user_id
    )
    .fetch_optional(&app_state.db)
    .await? else {
        return Ok(HttpResponse::NotFound().finish());
    };

	Ok(HttpResponse::Ok().json(user_friend_request_row!(row)))
}

pub async fn create_friend_request(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	path: web::Path<u64>,
) -> Result<impl Responder, BackendError> {
	let Some(user_id) = identity.has_scope(Scope::Friends(ReadWrite::Write)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let target_id = path.into_inner();

	if user_id == target_id {
		return Ok(HttpResponse::BadRequest().json(ErrorResponse {
			error: "Cannot send friend request to self".to_string(),
		}));
	}

	let Some(users) = query!(
        "SELECT Sender.username AS `sender_username`, Sender.display_name AS `sender_display_name`, Receiver.username AS `receiver_username`, Receiver.display_name AS `receiver_display_name` FROM User AS Sender, User AS Receiver WHERE Sender.id = ? AND Receiver.id = ?",
        user_id,
        target_id
    )
    .fetch_optional(&app_state.db)
    .await? else {
        return Ok(HttpResponse::NotFound().finish());
    };

	if query!(
		r#"
SELECT EXISTS(
    SELECT 1
    FROM (
        SELECT sender_id, receiver_id FROM UserFriendRequest
        UNION
        SELECT user_id AS `sender_id`, friend_id AS `receiver_id` FROM UserFriend
    ) AS Friend
    WHERE (sender_id, receiver_id) = (?, ?) OR (sender_id, receiver_id) = (?, ?)
) AS `exists: bool`"#,
		user_id,
		target_id,
		target_id,
		user_id
	)
	.fetch_one(&app_state.db)
	.await?
	.exists
	{
		return Ok(HttpResponse::Conflict().json(ErrorResponse {
			error: "Friend request already exists".to_string(),
		}));
	}

	let created_at = chrono::Utc::now();

	query!(
		"INSERT INTO UserFriendRequest (sender_id, receiver_id, created_at) VALUES (?, ?, ?)",
		user_id,
		target_id,
		created_at
	)
	.execute(&app_state.db)
	.await?;

	send_updates(
		[WsUpdateEvent::FriendRequestCreate(UserFriendRequest {
			sender: User {
				id: user_id,
				username: users.sender_username,
				display_name: users.sender_display_name,
			},
			receiver: User {
				id: target_id,
				username: users.receiver_username,
				display_name: users.receiver_display_name,
			},
			created_at,
		})],
		&app_state,
		[user_id, target_id],
	);

	Ok(HttpResponse::Created().finish())
}

pub async fn delete_friend_request(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	path: web::Path<u64>,
) -> Result<impl Responder, BackendError> {
	let Some(user_id) = identity.has_scope(Scope::Friends(ReadWrite::Write)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let target_id = path.into_inner();

	let Some(row) = query!(
        "SELECT sender_id, receiver_id FROM UserFriendRequest WHERE (sender_id, receiver_id) = (?, ?) OR (sender_id, receiver_id) = (?, ?)",
        user_id,
        target_id,
        target_id,
        user_id
    )
    .fetch_optional(&app_state.db)
    .await? else {
        return Ok(HttpResponse::NotFound().finish());
    };

	query!(
		"DELETE FROM UserFriendRequest WHERE sender_id = ? AND receiver_id = ?",
		row.sender_id,
		row.receiver_id
	)
	.execute(&app_state.db)
	.await?;

	send_updates(
		[WsUpdateEvent::FriendRequestDelete {
			sender_id: row.sender_id,
			receiver_id: row.receiver_id,
		}],
		&app_state,
		[user_id, target_id],
	);

	Ok(HttpResponse::Ok().finish())
}

pub async fn accept_friend_request(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
	path: web::Path<u64>,
	generator: web::Data<Mutex<snowflaked::Generator>>,
) -> Result<impl Responder, BackendError> {
	let Some(user_id) = identity.has_scope(Scope::Friends(ReadWrite::Write)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let target_id = path.into_inner();

	if user_id == target_id {
		return Ok(HttpResponse::BadRequest().json(ErrorResponse {
			error: "Cannot accept friend request from self".to_string(),
		}));
	}

	let Some(row) = query!(
        r#"SELECT UserFriendRequest.sender_id, UserFriendRequest.receiver_id,
Sender.username AS `sender_username`, Sender.display_name AS `sender_display_name`,
Receiver.username AS `receiver_username`, Receiver.display_name AS `receiver_display_name`
FROM UserFriendRequest
INNER JOIN User AS Sender ON UserFriendRequest.sender_id=Sender.id
INNER JOIN User AS Receiver ON UserFriendRequest.receiver_id=Receiver.id
WHERE (UserFriendRequest.sender_id, UserFriendRequest.receiver_id) = (?, ?) OR (UserFriendRequest.sender_id, UserFriendRequest.receiver_id) = (?, ?)
"#,
        target_id,
        user_id,
        user_id,
        target_id
    )
    .fetch_optional(&app_state.db)
    .await? else {
        return Ok(HttpResponse::NotFound().finish());
    };

	let mut tx = app_state.db.begin().await?;

	query!(
		r#"DELETE FROM UserFriendRequest WHERE (sender_id, receiver_id) = (?, ?)"#,
		row.sender_id,
		row.receiver_id
	)
	.execute(&mut *tx)
	.await?;

	let created_at = chrono::Utc::now();

	let result = query!(
		r#"INSERT INTO UserFriend (user_id, friend_id, created_at) VALUES (?, ?, ?)"#,
		row.sender_id,
		row.receiver_id,
		created_at
	)
	.execute(&mut *tx)
	.await;

	let channel_id = query!(r#"SELECT Channel.id
FROM Channel
INNER JOIN DMChannelRecipient AS Sender ON Sender.channel_id=Channel.id
INNER JOIN DMChannelRecipient AS Receiver ON Receiver.channel_id=Channel.id
WHERE (Sender.user_id, Receiver.user_id) = (?, ?) OR (Sender.user_id, Receiver.user_id) = (?, ?) AND Channel.kind = 'DM'
LIMIT 1
"#,
        row.sender_id,
        row.receiver_id,
        row.receiver_id,
        row.sender_id
    ).fetch_optional(&mut *tx).await?;

	let user = User {
		id: row.receiver_id,
		username: row.receiver_username,
		display_name: row.receiver_display_name,
	};

	let friend = User {
		id: row.sender_id,
		username: row.sender_username,
		display_name: row.sender_display_name,
	};

	let other_user = if user.id == user_id {
		friend.clone()
	} else {
		user.clone()
	};

	let channel = match channel_id {
		Some(row) => Channel {
			id: row.id,
			name: "".to_string(),
			kind: ChannelKind::DM,
			server_id: None,
			user: Some(other_user),
		},
		None => {
			let channel_id: u64 = {
				let mut generator = generator.lock().unwrap();
				generator.generate()
			};

			query!(
				"INSERT INTO Channel (id, name, kind, server_id) VALUES (?, '', 'DM', NULL)",
				channel_id
			)
			.execute(&mut *tx)
			.await?;

			query!(
				"INSERT INTO DMChannelRecipient (channel_id, user_id) VALUES (?, ?), (?, ?)",
				channel_id,
				row.sender_id,
				channel_id,
				row.receiver_id
			)
			.execute(&mut *tx)
			.await?;

			Channel {
				id: channel_id,
				server_id: None,
				kind: ChannelKind::DM,
				name: "".to_string(),
				user: Some(other_user),
			}
		}
	};

	tx.commit().await?;

	match result {
		Ok(_) => {
			send_updates(
				[
					WsUpdateEvent::FriendRequestDelete {
						sender_id: row.sender_id,
						receiver_id: row.receiver_id,
					},
					WsUpdateEvent::FriendCreate(UserFriend {
						user,
						friend,
						channel,
						created_at,
					}),
				],
				&app_state,
				[user_id, target_id],
			);

			Ok(HttpResponse::Ok().finish())
		}
		Err(err) => match err.as_database_error() {
			Some(err)
				if err
					.try_downcast_ref::<MySqlDatabaseError>()
					.is_some_and(|err| err.number() == 1062) =>
			{
				Ok(HttpResponse::Conflict().json(ErrorResponse {
					error: "Friendship already exists".to_string(),
				}))
			}
			_ => Err(err.into()),
		},
	}
}
