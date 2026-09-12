use crate::repos::channels::ChannelRepository;

pub mod models;
pub mod repos;

pub trait AppRepository: ChannelRepository {}
