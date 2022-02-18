use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use futures_util::{FutureExt, SinkExt, StreamExt};
use tokio::sync::{mpsc, RwLock};
use warp::{
    ws::{Message, WebSocket},
    Filter,
};

type Users = Arc<RwLock<HashMap<i32, tokio::sync::mpsc::UnboundedSender<Message>>>>;

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
    let (mut tx, mut rx) = websocket.split();
    let lock = users.write().await;

    let new_id = lock.len() as i32 + 1;

    tx.send(Message::text(format!("your id: {new_id}")))
        .await
        .unwrap();

    while let Some(message) = rx.next().await {
        match message {
            Ok(message) if message.is_text() => {
                println!("{new_id}: {message:?}");
                tx.send(Message::text("JP2 byl wielkim Polakiem"))
                    .await
                    .unwrap();
            }
            Ok(message) if message.is_close() => {
                tx.close().await.unwrap();
                break;
            }
            _ => (),
        }
    }
    println!("socket to {new_id} closed");
}
