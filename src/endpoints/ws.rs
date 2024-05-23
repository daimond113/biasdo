use std::{
    collections::hash_map::Entry as StdEntry,
    sync::Mutex,
    time::{Duration, Instant},
};

use actix_web::{
    rt::{self, time::interval},
    web, HttpRequest, Responder,
};
use actix_ws::{CloseCode, CloseReason, Message};
use dashmap::mapref::entry::Entry;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::query;
use tokio::select;

use crate::{
    middleware::{get_identity, Identity},
    AppState,
};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(10);

// should be set to the greatest common divisor of AUTHENTICATION_TIMEOUT and REAUTHENTICATION_INTERVAL
const AUTHENTICATION_INTERVAL: Duration = Duration::from_secs(10);
const AUTHENTICATION_TIMEOUT: Duration = Duration::from_secs(10);
const REAUTHENTICATION_INTERVAL: Duration = Duration::from_secs(600);

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
enum WsMessage {
    Authenticate(String),
    Reauthenticate,
}

async fn ws_handler(
    mut session: actix_ws::Session,
    mut msg_stream: actix_ws::MessageStream,
    app_state: web::Data<AppState>,
    session_id: u64,
) {
    let connected_at = Instant::now();
    let mut last_heartbeat = Instant::now();
    let mut heartbeat_interval = interval(HEARTBEAT_INTERVAL);

    let mut auth_info = None;
    let mut auth_interval = interval(AUTHENTICATION_INTERVAL);

    let mut last_reauthentication = Instant::now() + REAUTHENTICATION_INTERVAL;
    let mut reauth_requested = false;

    let reason = loop {
        select! {
            msg = msg_stream.next() => {
                let msg = match msg {
                    Some(Ok(msg)) => msg,
                    _ => break Some(CloseReason {
                        code: CloseCode::Invalid,
                        description: None,
                    }),
                };

                match msg {
                    Message::Close(reason) => {
                        break reason;
                    }

                    Message::Ping(bytes) => {
                        last_heartbeat = Instant::now();
                        let _ = session.pong(&bytes).await;
                    }

                    Message::Pong(_) => {
                        last_heartbeat = Instant::now();
                    }

                    Message::Text(text) => {
                        let Ok(msg) = serde_json::from_str::<WsMessage>(&text) else {
                            continue;
                        };

                        #[allow(clippy::single_match)]
                        match msg {
                            WsMessage::Authenticate(token) => {
                                if auth_info.is_some() && !reauth_requested {
                                    break Some(CloseReason {
                                        code: CloseCode::Policy,
                                        description: Some("Cannot reauthenticate".to_string()),
                                    });
                                }

                                let (user_id, scopes) = match get_identity(&token, &app_state).await {
                                    Ok(Some((Identity::User(identity), _))) => identity,
                                    _ => {
                                        break Some(CloseReason {
                                            code: CloseCode::Policy,
                                            description: Some("Unauthorized".to_string()),
                                        });
                                    }
                                };

                                if let Some(previous_auth) = auth_info {
                                    if previous_auth != user_id {
                                        break Some(CloseReason {
                                            code: CloseCode::Policy,
                                            description: Some("Unauthorized".to_string()),
                                        });
                                    }

                                    if let Some(mut conns) = app_state.user_connections.get_mut(&user_id) {
                                        if let StdEntry::Occupied(mut session_info) = conns.entry(session_id) {
                                            match (&session_info.get().0, scopes) {
                                                (Some(og_scopes), Some(scopes))
                                                    if !scopes.difference(og_scopes).collect::<Vec<_>>().is_empty() =>
                                                {
                                                    break Some(CloseReason {
                                                        code: CloseCode::Policy,
                                                        description: Some("Unauthorized".to_string()),
                                                    });
                                                }
                                                (_, scopes) => {
                                                    session_info.insert((scopes, session.clone()));
                                                }
                                            }
                                        }
                                    }

                                    last_reauthentication = Instant::now();
                                    reauth_requested = false;
                                } else {
                                    app_state
                                        .user_connections
                                        .entry(user_id)
                                        .or_default()
                                        .insert(session_id, (scopes, session.clone()));

                                    let Ok(servers) = query!(
                                        "SELECT server_id FROM ServerMember WHERE user_id = ?",
                                        user_id
                                    )
                                    .fetch_all(&app_state.db)
                                    .await
                                    else {
                                        break Some(CloseReason {
                                            code: CloseCode::Error,
                                            description: None,
                                        });
                                    };

                                    for row in servers {
                                        app_state
                                            .server_connections
                                            .entry(row.server_id)
                                            .or_default()
                                            .insert(user_id);
                                    }

                                    auth_info = Some(user_id);
                                }
                            }
                            _ => {}
                        }
                    }

                    _ => {}
                };
            }

            _ = heartbeat_interval.tick() => {
                if Instant::now().duration_since(last_heartbeat) > HEARTBEAT_TIMEOUT {
                    break Some(CloseReason {
                        code: CloseCode::Policy,
                        description: Some("Heartbeat timeout".to_string()),
                    });
                }

                let _ = session.ping(b"").await;
            }

            _ = auth_interval.tick() => {
                if auth_info.is_some() {
                    if Instant::now().duration_since(last_reauthentication) > REAUTHENTICATION_INTERVAL && !reauth_requested {
                        let _ = session.text(serde_json::to_string(&WsMessage::Reauthenticate).unwrap()).await;
                        reauth_requested = true;
                    } else if Instant::now().duration_since(last_reauthentication) > AUTHENTICATION_TIMEOUT && reauth_requested {
                        break Some(CloseReason {
                            code: CloseCode::Policy,
                            description: Some("Unauthorized".to_string()),
                        });
                    }
                } else if Instant::now().duration_since(connected_at) > AUTHENTICATION_TIMEOUT {
                    break Some(CloseReason {
                        code: CloseCode::Policy,
                        description: Some("Unauthorized".to_string()),
                    });
                }
            }
        }
    };

    if let Some(user_id) = auth_info {
        if let Entry::Occupied(mut user_connections) = app_state.user_connections.entry(user_id) {
            if user_connections.get().len() == 1 {
                user_connections.remove_entry();
            } else {
                user_connections.get_mut().remove(&session_id);
            }
        }

        if let Ok(servers) = query!(
            "SELECT server_id FROM ServerMember WHERE user_id = ?",
            user_id
        )
        .fetch_all(&app_state.db)
        .await
        {
            for server_id in servers.into_iter().map(|row| row.server_id) {
                if let Entry::Occupied(mut server_connections) =
                    app_state.server_connections.entry(server_id)
                {
                    if server_connections.get().len() == 1 {
                        server_connections.remove_entry();
                    } else {
                        server_connections.get_mut().remove(&user_id);
                    }
                }
            }
        }
    }

    let _ = session.close(reason).await;
}

pub async fn ws(
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<AppState>,
    generator: web::Data<Mutex<snowflaked::Generator>>,
) -> Result<impl Responder, actix_web::Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

    rt::spawn(ws_handler(
        session,
        msg_stream,
        app_state,
        generator.lock().unwrap().generate(),
    ));

    Ok(res)
}
