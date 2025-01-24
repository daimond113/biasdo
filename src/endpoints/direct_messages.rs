use crate::{
	error::Error,
	middleware::Identity,
	models::{
		channel::{Channel, ChannelKind},
		scope::{HasScope, ReadWrite, Scope},
		user::User,
	},
	AppState,
};
use actix_web::{web, HttpResponse, Responder};
use sqlx::query;

pub async fn get_direct_channels(
	identity: web::ReqData<Identity>,
	app_state: web::Data<AppState>,
) -> Result<impl Responder, Error> {
	let Some(user_id) = identity.has_scope(Scope::Friends(ReadWrite::Read)) else {
		return Ok(HttpResponse::Forbidden().finish());
	};

	let channels = query!(
        r#"SELECT Channel.id,
User.id AS user_id, User.username, User.display_name
FROM DMChannelRecipient
INNER JOIN Channel ON DMChannelRecipient.channel_id=Channel.id AND Channel.kind='DM'
INNER JOIN DMChannelRecipient AS OtherRecipient ON Channel.id=OtherRecipient.channel_id AND OtherRecipient.user_id!=DMChannelRecipient.user_id
INNER JOIN User ON OtherRecipient.user_id=User.id
WHERE DMChannelRecipient.user_id = ?
"#,
        user_id
    )
    .fetch_all(&app_state.db)
    .await?;

	Ok(HttpResponse::Ok().json(
		channels
			.into_iter()
			.map(|row| Channel {
				id: row.id,
				name: "".to_string(),
				kind: ChannelKind::DM,
				server_id: None,
				user: Some(User {
					id: row.user_id,
					username: row.username,
					display_name: row.display_name,
				}),
			})
			.collect::<Vec<_>>(),
	))
}
