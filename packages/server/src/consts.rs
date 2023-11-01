use actix_web::web;
use serde_json::Value;
use crate::AppState;
use crate::ws::JsonMessage;

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
) where T: serde::Serialize + std::fmt::Debug + Clone + Send + 'static {
    if let Some(server_connections) = data
        .server_connections
        .read()
        .unwrap()
        .get(&server_id) {
        let user_connections = data
            .user_connections
            .read()
            .unwrap();

        for user_id in server_connections {
            if let Some(user_sockets) = user_connections.get(&user_id) {
                user_sockets.iter().for_each(|addr| {
                    addr.do_send(message.clone());
                });
            }
        }
    }
}