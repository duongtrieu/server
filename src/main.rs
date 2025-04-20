use warp::Filter;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use warp::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};

type Clients = Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<Message>>>>;

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    let chat_route = warp::path("ws")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .map(|ws: warp::ws::Ws, clients| {
            ws.on_upgrade(move |socket| client_connected(socket, clients))
        });

    warp::serve(chat_route).run(([0, 0, 0, 0], 3030)).await;
}

fn with_clients(
    clients: Clients,
) -> impl Filter<Extract = (Clients,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

async fn client_connected(ws: WebSocket, clients: Clients) {
    let (mut tx, mut rx) = ws.split();
    let (client_tx, mut client_rx) = tokio::sync::mpsc::unbounded_channel();
    let id = Uuid::new_v4().to_string();
    println!("Client connected: {}", id);

    // Register the client immediately
    clients.lock().unwrap().insert(id.clone(), client_tx);

    // Spawn a task to handle outgoing messages
    tokio::spawn(async move {
        while let Some(message) = client_rx.recv().await {
            if tx.send(message).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(Ok(msg)) = rx.next().await {
        if msg.is_text() {
            let message = msg.to_str().unwrap();
            let peers = clients.lock().unwrap();
            for (peer_id, sender) in peers.iter() {
                if peer_id != &id {
                    if let Err(e) = sender.send(Message::text(message)) {
                        eprintln!("Error sending message to peer {}: {}", peer_id, e);
                    }
                }
            }
        }
    }

    // Remove the client when disconnected
    clients.lock().unwrap().remove(&id);
    println!("Client disconnected: {}", id);
}