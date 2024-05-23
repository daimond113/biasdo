use serde::{Deserialize, Deserializer};

pub mod channels;
pub mod direct_messages;
pub mod friend_requests;
pub mod friends;
pub mod invites;
pub mod members;
pub mod messages;
pub mod oauth;
pub mod servers;
pub mod users;
pub mod ws;

#[macro_export]
macro_rules! update_structure {
    (raw $tbl_name:expr, $body:expr, $($name:ident),+) => {{
        use sqlx::{QueryBuilder, MySql};
        let mut query_builder: QueryBuilder<MySql> = QueryBuilder::new(concat!("UPDATE ", $tbl_name, " SET "));
        let mut pushed = false;

        $(
            if let Some(value) = &$body.$name {
                if pushed {
                    query_builder.push(", ");
                }
                pushed = true;
                query_builder
                    .push(concat!(stringify!($name), " = "))
                    .push_bind(value);
            }
        )+

        (pushed, query_builder)
    }};
    ($tbl_name:expr, $body:expr, $($name:ident),+) => {{
        let (pushed, query_builder) = update_structure!(raw $tbl_name, $body, $($name),+);

        if !pushed {
            return Ok(HttpResponse::BadRequest().finish());
        }

        query_builder
    }};
}

pub fn trim_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.trim().to_string())
}

pub fn trim_opt_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    Ok(s.map(|s| s.trim().to_string()))
}

// https://github.com/serde-rs/serde/issues/984
pub fn deserialize_some_trimmed<'de, D>(deserializer: D) -> Result<Option<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer)
        .map(|s: Option<String>| s.map(|s| s.to_string().trim().to_string()))
        .map(Some)
}
