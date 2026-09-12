use biasdo_core::AppRepository;
use sqlx::{Database as _, MySql, MySqlPool};

mod repos;

pub struct MariaDBBackend {
	pool: MySqlPool,
}

impl MariaDBBackend {
	pub const URL_SCHEMES: &'static [&'static str] = MySql::URL_SCHEMES;

	pub async fn connect(url: &str) -> Result<Self, sqlx::Error> {
		let pool = MySqlPool::connect(url).await?;
		sqlx::migrate!().run(&pool).await?;
		Ok(Self { pool })
	}
}

impl AppRepository for MariaDBBackend {}
