mod endpoints;
mod error;
mod middleware;
mod models;
mod ws;

use crate::{middleware::TokenKey, models::scope::Scope};
use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{
	middleware::{from_fn, Compress, Condition, Logger, NormalizePath, TrailingSlash},
	rt::System,
	web, App, HttpServer,
};
use dashmap::DashMap;
use log::info;
use snowflaked::Generator;
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use std::{
	collections::{HashMap, HashSet},
	hash::{DefaultHasher, Hash, Hasher},
	sync::Mutex,
	time::{Duration, UNIX_EPOCH},
};

type Session = (Option<HashSet<Scope>>, actix_ws::Session);

pub struct AppState {
	pub db: MySqlPool,
	// server id -> user id(s)
	pub server_connections: DashMap<u64, HashSet<u64>>,
	// user id -> ws(s) // multiple sessions
	pub user_connections: DashMap<u64, HashMap<u64, Session>>,
}

#[macro_export]
macro_rules! benv {
    ($name:expr) => {
        std::env::var($name)
    };
    ($name:expr => $default:expr) => {
        benv!($name).unwrap_or($default.to_string())
    };
    (required $name:expr) => {
        benv!($name).expect(concat!("Environment variable `", $name, "` must be set"))
    };
    (parse $name:expr) => {
        benv!($name)
            .map(|v| v.parse().expect(concat!(
                "Environment variable `",
                $name,
                "` must be a valid value"
            )))
    };
    (parse required $name:expr) => {
        benv!(parse $name).expect(concat!("Environment variable `", $name, "` must be set"))
    };
    (parse $name:expr => $default:expr) => {
        benv!($name => $default)
            .parse()
            .expect(concat!(
                "Environment variable `",
                $name,
                "` must a valid value"
            ))
    };
}

async fn run(with_sentry: bool) -> std::io::Result<()> {
	let address = benv!("ADDRESS" => "127.0.0.1");
	let port: u16 = benv!(parse "PORT" => "8080");

	let db_url = benv!(required "DATABASE_URL");

	let pool = MySqlPoolOptions::new()
		.max_connections(5)
		.connect(&db_url)
		.await
		.expect("Failed to connect to database");

	sqlx::migrate!()
		.run(&pool)
		.await
		.expect("Failed to run migrations");

	let app_data = web::Data::new(AppState {
		db: pool,
		server_connections: DashMap::new(),
		user_connections: DashMap::new(),
	});

	let generic_governor_config = GovernorConfigBuilder::default()
		.key_extractor(TokenKey)
		.burst_size(250)
		.requests_per_second(50)
		.use_headers()
		.finish()
		.unwrap();

	info!("listening on {address}:{port}");

	HttpServer::new(move || {
		let mut hasher = DefaultHasher::new();
		if let Ok(machine_id) = benv!("FLY_MACHINE_ID") {
			machine_id.hash(&mut hasher);
		}
		std::thread::current().id().hash(&mut hasher);
		// the max instance is 2^10-1
		let instance = (hasher.finish() % 1024) as u16;

		App::new()
			.wrap(Condition::new(with_sentry, sentry_actix::Sentry::new()))
			.wrap(NormalizePath::new(TrailingSlash::Trim))
			.wrap(Cors::permissive())
			.wrap(Logger::default())
			.wrap(Compress::default())
			.app_data(app_data.clone())
			.app_data(web::Data::new(Mutex::new(
				Generator::builder()
					.instance(instance)
					.epoch(UNIX_EPOCH + Duration::from_secs(1716501600))
					.build::<Generator>(),
			)))
			.route(
				"/",
				web::get().to(|| async {
					concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"))
				}),
			)
			.service(
				web::scope("/v0")
					.route("/register", web::post().to(endpoints::users::register_user))
					.route("/login", web::post().to(endpoints::users::login_user))
					.route(
						"/logout",
						web::post()
							.to(endpoints::users::logout_user)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.service(
						web::resource("/clients")
							.get(endpoints::oauth::clients::get_clients)
							.post(endpoints::oauth::clients::register_client)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.service(
						web::resource("/clients/authorize")
							.get(endpoints::oauth::authorization::get_authorize_info)
							.post(endpoints::oauth::authorization::authorize_client)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.service(
						web::resource("/clients/{client_id}")
							.get(endpoints::oauth::clients::get_client)
							.patch(endpoints::oauth::clients::update_client)
							.delete(endpoints::oauth::clients::delete_client)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.route(
						"/friend-requests",
						web::get()
							.to(endpoints::friend_requests::get_friend_requests)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.service(
						web::resource("/users/{user_id}/friend-request")
							.get(endpoints::friend_requests::get_friend_request)
							.post(endpoints::friend_requests::create_friend_request)
							.delete(endpoints::friend_requests::delete_friend_request)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.route(
						"/users/{user_id}/friend-request/accept",
						web::post()
							.to(endpoints::friend_requests::accept_friend_request)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.route(
						"/friends",
						web::get()
							.to(endpoints::friends::get_friends)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.service(
						web::resource("/friends/{user_id}")
							.get(endpoints::friends::get_friend)
							.delete(endpoints::friends::delete_friend)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.route(
						"/direct-channels",
						web::get()
							.to(endpoints::direct_messages::get_direct_channels)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.service(
						web::resource("/users/@me")
							.get(endpoints::users::get_current_user)
							.patch(endpoints::users::update_user)
							.delete(endpoints::users::delete_user)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.route(
						"/users/{user_id}",
						web::get()
							.to(endpoints::users::get_user)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.route(
						"/users/username/{user_id}",
						web::get()
							.to(endpoints::users::get_user_by_username)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.service(
						web::resource("/servers")
							.get(endpoints::servers::get_servers)
							.post(endpoints::servers::create_server)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.service(
						web::resource("/servers/{server_id}")
							.get(endpoints::servers::get_server)
							.patch(endpoints::servers::update_server)
							.delete(endpoints::servers::delete_server)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.route(
						"/servers/{server_id}/leave",
						web::post()
							.to(endpoints::servers::leave_server)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.service(
						web::resource("/servers/{server_id}/channels")
							.get(endpoints::channels::get_channels)
							.post(endpoints::channels::create_channel)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.service(
						web::resource("/servers/{server_id}/channels/{channel_id}")
							.get(endpoints::channels::get_channel)
							.patch(endpoints::channels::update_channel)
							.delete(endpoints::channels::delete_channel)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.service(
						web::resource("/channels/{channel_id}/messages")
							.get(endpoints::messages::get_messages)
							.post(endpoints::messages::create_message)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.service(
						web::resource("/channels/{channel_id}/messages/{message_id}")
							.get(endpoints::messages::get_message)
							.patch(endpoints::messages::update_message)
							.delete(endpoints::messages::delete_message)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.service(
						web::resource("/servers/{server_id}/invites")
							.get(endpoints::invites::get_invites)
							.post(endpoints::invites::create_invite)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.route(
						"/servers/{server_id}/members",
						web::get()
							.to(endpoints::members::get_members)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.service(
						web::resource("/servers/{server_id}/members/{user_id}")
							.get(endpoints::members::get_member)
							.patch(endpoints::members::update_member)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.route(
						"/servers/{server_id}/invites/{invite_id}",
						web::delete()
							.to(endpoints::invites::delete_invite)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.service(
						web::resource("/invites/{invite_id}")
							.get(endpoints::invites::get_invite)
							.post(endpoints::invites::accept_invite)
							.wrap(Governor::new(&generic_governor_config))
							.wrap(from_fn(middleware::authentication)),
					)
					.route(
						"/token",
						web::post()
							.to(endpoints::oauth::token::exchange_token)
							.wrap(from_fn(middleware::maybe_authentication)),
					)
					.route("/ws", web::get().to(endpoints::ws::ws)),
			)
	})
	.bind((address, port))?
	.run()
	.await
}

// can't use #[actix_web::main] because of Sentry:
// "Note: Macros like #[tokio::main] and #[actix_web::main] are not supported. The Sentry client must be initialized before the async runtime is started so that all threads are correctly connected to the Hub."
// https://docs.sentry.io/platforms/rust/guides/actix-web/
fn main() -> std::io::Result<()> {
	dotenvy::dotenv().ok();

	let sentry_url = benv!("SENTRY_URL").ok();
	let with_sentry = sentry_url.is_some();

	let mut log_builder = pretty_env_logger::formatted_builder();
	log_builder.parse_env(pretty_env_logger::env_logger::Env::default().default_filter_or("info"));

	if with_sentry {
		let logger = sentry::integrations::log::SentryLogger::with_dest(log_builder.build());
		log::set_boxed_logger(Box::new(logger)).unwrap();
		log::set_max_level(log::LevelFilter::Info);
	} else {
		log_builder.try_init().unwrap();
	}

	let _guard = if let Some(sentry_url) = sentry_url {
		std::env::set_var("RUST_BACKTRACE", "1");

		Some(sentry::init((
			sentry_url,
			sentry::ClientOptions {
				release: sentry::release_name!(),
				..Default::default()
			},
		)))
	} else {
		None
	};

	System::new().block_on(run(with_sentry))
}
