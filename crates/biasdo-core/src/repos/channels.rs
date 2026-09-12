use async_trait::async_trait;
use futures_core::stream::BoxStream;

use crate::models::*;

#[derive(Debug)]
pub enum CreateServerChannelOutcome {
	NoServer,
	LimitReached,
	Created,
}

#[async_trait]
pub trait ChannelRepository {
	async fn create_server_channel(
		&self,
		server_id: &ServerId,
		channel_id: &ChannelId,
		user_id: &UserId,
		name: &str,
	) -> anyhow::Result<CreateServerChannelOutcome>;

	async fn get_server_channels(
		&self,
		server_id: &ServerId,
		user_id: &UserId,
	) -> anyhow::Result<BoxStream<'static, Channel>>;

	async fn get_server_channel(
		&self,
		server_id: &ServerId,
		channel_id: &ChannelId,
		user_id: &UserId,
	) -> anyhow::Result<Option<Channel>>;

	async fn update_server_channel(
		&self,
		server_id: &ServerId,
		channel_id: &ChannelId,
		user_id: &UserId,
		name: &str,
	) -> anyhow::Result<bool>;

	async fn delete_server_channel(
		&self,
		server_id: &ServerId,
		channel_id: &ChannelId,
		user_id: &UserId,
	) -> anyhow::Result<bool>;
}
