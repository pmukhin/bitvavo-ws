use bitvavo_tungstenite::event::AuthRequest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, LinkedList};
use std::net::TcpListener;
use std::sync::{Arc, RwLock};
use std::thread::spawn;
use tungstenite::accept;

#[derive(Debug, Deserialize)]
struct SubscriptionRequest {
    action: String,
    channels: Vec<Subscription>,
}

#[derive(Debug, Deserialize)]
struct Subscription {
    markets: Vec<String>,
    name: String,
}

struct UserStorage {
    users: HashMap<&'static str, &'static str>,
}

impl UserStorage {
    pub fn new() -> Self {
        Self {
            users: HashMap::from([("xxx_yyyy", "00000000-0000-0000-0000-000000000001")]),
        }
    }

    pub fn tok_to_guid(&self, token: String) -> Option<&'static str> {
        self.users.get(token.as_str()).copied()
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    pub action: String,
}

fn main() {
    env_logger::init();
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();
    let connections = Arc::new(RwLock::new(HashMap::new()));
    let user_storage = Arc::new(RwLock::new(UserStorage::new()));

    for stream in server.incoming() {
        let conn = Arc::clone(&connections);
        let us = Arc::clone(&user_storage);

        log::info!("spawning a new connection-thread");

        spawn(move || {
            let mut authenticated = false;
            let mut websocket = accept(stream.unwrap()).unwrap();

            loop {
                match websocket.read() {
                    Ok(tungstenite::Message::Ping(bytes)) => {
                        websocket.write(tungstenite::Message::Pong(bytes)).unwrap()
                    }

                    Ok(tungstenite::Message::Text(bytes)) => {
                        let request = serde_json::from_slice::<Request>(bytes.as_ref());
                        if request.is_err() {
                            break;
                        }
                        match request.unwrap().action.as_str() {
                            "privateGetBalance" => {
                                if !authenticated {
                                    break;
                                }
                                let bytes = json!({
                                    "action": "privateGetBalance",
                                    "response": [{
                                        "symbol": "BTC",
                                        "available": "1.57593193",
                                        "inOrder": "0.00",
                                    }, {
                                        "symbol": "EUR",
                                        "available": "214232.00",
                                        "inOrder": "0.00",
                                    }]
                                })
                                .to_string();
                                websocket
                                    .write(tungstenite::Message::Text(bytes.into()))
                                    .unwrap();
                                continue;
                            }
                            "authenticate" => {
                                if authenticated {
                                    // ignore
                                    continue;
                                }
                                let maybe_guid =
                                    serde_json::from_slice::<AuthRequest>(bytes.as_ref())
                                        .ok()
                                        .and_then(|req| us.write().unwrap().tok_to_guid(req.key));
                                match maybe_guid {
                                    Some(guid) => {
                                        conn.write()
                                            .unwrap()
                                            .insert(guid, LinkedList::<serde_json::Value>::new());
                                        authenticated = true;

                                        let authenticated =
                                            json!({ "event": "authenticate" }).to_string();

                                        websocket
                                            .write(tungstenite::Message::Text(authenticated.into()))
                                            .unwrap();
                                        log::info!(
                                            "authenticated user : {}, confirmation sent",
                                            guid
                                        );
                                        continue;
                                    }
                                    None => {
                                        log::info!("authentication failed");
                                        break;
                                    }
                                }
                            }
                            "subscribe" => {
                                let request_result =
                                    serde_json::from_slice::<SubscriptionRequest>(bytes.as_ref());
                                if request_result.is_err() {
                                    log::error!(
                                        "subscription failed: {}",
                                        request_result.unwrap_err()
                                    );
                                    break;
                                }
                                let subscribed = json!({ "event": "subscribe" }).to_string();
                                websocket
                                    .write(tungstenite::Message::Text(subscribed.into()))
                                    .unwrap();
                            }
                            action => {
                                log::info!("unknown action: {}", action);
                                break;
                            }
                        }
                    }
                    somethings_else => {
                        log::error!("somethings_else: {:?}", somethings_else);
                        break;
                    }
                }
            }
            log::info!("connection terminated :(");
            websocket.close(None).unwrap();
        });
    }
}
