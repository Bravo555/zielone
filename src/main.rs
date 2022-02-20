use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use futures_util::{SinkExt, StreamExt};
use tokio::sync::RwLock;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};

#[derive(Debug, Copy, Clone)]
enum UserFeedback {
    Green,
    Red,
}

impl ToString for UserFeedback {
    fn to_string(&self) -> String {
        match self {
            UserFeedback::Green => "zielone".to_string(),
            UserFeedback::Red => "czerwone".to_string(),
        }
    }
}

#[derive(Debug)]
struct User {
    tx: tokio::sync::mpsc::UnboundedSender<Message>,
    feeback: Option<UserFeedback>,
}

type Users = Arc<RwLock<HashMap<usize, User>>>;

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

#[tokio::main]
async fn main() {
    let users: Users = Arc::new(RwLock::new(HashMap::new()));

    let users_filter = warp::any().map(move || users.clone());

    let ws = warp::path("ws")
        // The `ws()` filter will prepare the Websocket handshake.
        .and(warp::ws())
        .and(users_filter)
        .map(|ws: warp::ws::Ws, users: Users| {
            // And then our closure will be called when it completes...
            ws.on_upgrade(move |websocket| user_connected(websocket, users))
        });

    let files = warp::get().and(warp::fs::dir("./static/"));

    let routes = ws.or(files);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn user_connected(websocket: WebSocket, users: Users) {
    let (mut ws_tx, mut ws_rx) = websocket.split();
    let (chan_tx, mut chan_rx) = tokio::sync::mpsc::unbounded_channel();

    let new_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    let mut lock = users.write().await;

    let user = User {
        tx: chan_tx.clone(),
        feeback: None,
    };

    lock.insert(new_id, user);
    drop(lock);

    ws_tx
        .send(Message::text(format!("your id: {new_id}")))
        .await
        .unwrap();

    broadcast_user_list(&users).await;

    tokio::spawn(async move {
        while let Some(message) = chan_rx.recv().await {
            ws_tx.send(message).await.unwrap_or_else(|e| {
                eprintln!("websocket send error: {}", e);
            });
        }
    });

    while let Some(message) = ws_rx.next().await {
        match message {
            Ok(message) if message.is_text() => {
                let text = message.to_str().unwrap();
                let users = users.clone();

                match text {
                    "green" => {
                        users.write().await.get_mut(&new_id).unwrap().feeback =
                            Some(UserFeedback::Green);
                        broadcast_user_list(&users).await;

                        tokio::spawn(async move {
                            tokio::time::sleep(Duration::from_secs(10)).await;
                            users
                                .clone()
                                .write()
                                .await
                                .get_mut(&new_id)
                                .unwrap()
                                .feeback = None;
                            broadcast_user_list(&users).await;
                        });
                    }
                    "red" => {
                        users.write().await.get_mut(&new_id).unwrap().feeback =
                            Some(UserFeedback::Red);
                        broadcast_user_list(&users).await;

                        tokio::spawn(async move {
                            tokio::time::sleep(Duration::from_secs(10)).await;
                            users
                                .clone()
                                .write()
                                .await
                                .get_mut(&new_id)
                                .unwrap()
                                .feeback = None;
                            broadcast_user_list(&users).await;
                        });
                    }
                    "2137" => {
                        chan_tx
                            .send(Message::text("JP2 byl wielkim Polakiem"))
                            .unwrap();
                    }
                    _ => (),
                }
            }
            Ok(message) if message.is_close() => {
                break;
            }
            _ => (),
        }
    }

    users.write().await.remove(&new_id);
    println!("socket to {new_id} closed");

    broadcast_user_list(&users).await;
}

async fn broadcast_user_list(users: &Users) {
    let users_string = users
        .read()
        .await
        .iter()
        .map(|(k, v)| {
            let value_string = v
                .feeback
                .as_ref()
                .map(|feedback| format!(" - {}", feedback.to_string()))
                .unwrap_or("".to_string());
            let key = k.to_string();
            format!("{key}{value_string}")
        })
        .fold(String::new(), |a, b| a + &b + "\n");

    for user in users.read().await.values() {
        user.tx
            .send(Message::text(format!("list of users:\n{users_string}")))
            .unwrap();
    }
}
