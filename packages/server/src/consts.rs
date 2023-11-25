use crate::ws::JsonMessage;
use crate::AppState;
use actix_web::web;
use serde_json::Value;

pub const SESSION_COOKIE_NAME: &str = "biasdo_SESSION";

pub fn merge_json(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                merge_json(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}

pub fn send_to_server_members<T>(
    data: &web::Data<AppState>,
    server_id: u64,
    message: &JsonMessage<T>,
) where
    T: serde::Serialize + std::fmt::Debug + Clone + Send + 'static,
{
    if let Some(server_connections) = data.server_connections.get(&server_id) {
        for user_id in server_connections.iter() {
            if let Some(user_sockets) = data.user_connections.get(&user_id) {
                user_sockets.iter().for_each(|addr| {
                    addr.do_send(message.clone());
                });
            }
        }
    }
}

#[macro_export]
macro_rules! add_value_to_json {
    ($into:expr, $thing:expr, $key:expr) => {{
        let mut into_value = serde_json::to_value($into.clone()).unwrap();
        let thing_value = serde_json::to_value($thing.clone()).unwrap();

        let mut map = serde_json::Map::new();
        map.insert($key.to_string(), thing_value);

        crate::consts::merge_json(&mut into_value, &serde_json::Value::Object(map));

        into_value
    }};
}
