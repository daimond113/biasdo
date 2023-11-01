use std::collections::HashSet;
use std::hash::Hash;
use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web::{Error, get, HttpRequest, HttpResponse, web};
use actix_web_actors::ws;
use cuid2::create_id;
use sqlx::query;

use crate::AppState;
use crate::structures::session::Session;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct MyWebSocket {
    hb: Instant,
    app_data: web::Data<AppState>,
    session: Session,
    pub unique_session_id: String,
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct JsonMessage<T>(pub T)
    where
        T: serde::Serialize + std::fmt::Debug + Clone + Send + 'static;

impl Hash for MyWebSocket {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.unique_session_id.hash(state);
        self.session.hash(state);
    }
}

impl PartialEq for MyWebSocket {
    fn eq(&self, other: &Self) -> bool {
        self.unique_session_id == other.unique_session_id && self.session == other.session
    }
}

impl Eq for MyWebSocket {}

async fn on_started(app_data: web::Data<AppState>, session: Session, addr: Addr<MyWebSocket>) {
    let mut user_connections = app_data.user_connections.write().unwrap();
    user_connections
        .entry(session.user_id.0)
        .or_insert_with(HashSet::new)
        .insert(addr);

    let ids = query!(
                "SELECT server_id FROM Member WHERE user_id = ?",
                session.user_id.0
            )
        .fetch_all(&app_data.db)
        .await
        .unwrap();

    let mut server_connections = app_data.server_connections.write().unwrap();

    for id in ids {
        server_connections
            .entry(id.server_id)
            .or_insert_with(HashSet::new)
            .insert(session.user_id.0);
    }
}

async fn on_stopped(app_data: web::Data<AppState>, session: Session, addr: Addr<MyWebSocket>) {
    let mut user_connections = app_data.user_connections.write().unwrap();
    let current_connections = user_connections.get_mut(&session.user_id.0).unwrap();

    if current_connections.len() == 1 {
        user_connections.remove(&session.user_id.0);

        let ids = query!(
                "SELECT server_id FROM Member WHERE user_id = ?",
                session.user_id.0
            )
            .fetch_all(&app_data.db)
            .await
            .unwrap();

        let mut server_connections = app_data.server_connections.write().unwrap();

        for id in ids {
            if let Some(server) = server_connections.get_mut(&id.server_id) {
                if server.is_empty() {
                    server_connections.remove(&id.server_id);
                } else {
                    server.remove(&session.user_id.0);
                }
            }
        }
    } else {
        current_connections.remove(&addr);
    }
}

impl MyWebSocket {
    pub fn new(data: &web::Data<AppState>, session: &Session) -> Self {
        Self { hb: Instant::now(), app_data: web::Data::clone(data), session: session.clone(), unique_session_id: create_id() }
    }

    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Websocket Client heartbeat failed, disconnecting!");

                ctx.stop();

                return;
            }

            let exp_diff = chrono::Utc::now() - act.session.expires_at;

            if exp_diff > chrono::Duration::seconds(0) {
                println!("Websocket Client session expired, disconnecting!");

                ctx.stop();

                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        actix_web::rt::spawn(on_started(web::Data::clone(&self.app_data), self.session.clone(), ctx.address()));
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        actix_web::rt::spawn(on_stopped(web::Data::clone(&self.app_data), self.session.clone(), ctx.address()));
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl<T: serde::Serialize + std::fmt::Debug + Clone + Send + 'static> Handler<JsonMessage<T>> for MyWebSocket {
    type Result = ();

    fn handle(&mut self, msg: JsonMessage<T>, ctx: &mut Self::Context) {
        ctx.text(serde_json::to_string(&msg.0).unwrap());
    }
}

#[get("/ws")]
pub async fn ws_route(req: HttpRequest, stream: web::Payload, app_data: web::Data<AppState>, session: web::ReqData<Session>) -> Result<HttpResponse, Error> {
    ws::start(MyWebSocket::new(&app_data, &session), &req, stream)
}