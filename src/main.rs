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
    let (tx, mut rx) = ws.split();
    let (client_tx, mut client_rx) = tokio::sync::mpsc::unbounded_channel();
    let id = uuid::Uuid::new_v4().to_string();
    println!("Client connected: {}", id);
    let clients_clone = clients.clone();
    let id_clone = id.clone();
    let tx = tokio::spawn(async move {
        while let Some(message) = client_rx.recv().await {
            clients_clone.lock().unwrap().insert(id_clone.clone(), client_tx.clone());
            let _ = client_tx.send(message.into());
            break;
        }
    });

    while let Some(Ok(msg)) = rx.next().await {
        if msg.is_text() {
            let message = msg.to_str().unwrap();
            let peers = clients.lock().unwrap();
            for (peer_id, sender) in peers.iter() {
                if peer_id != &id {
                    if let Err(e) = sender.send(Message::text(message)) {
                        eprintln!("Failed to send message to peer: {}", e);
                    }
                }
            }
        }
    }

    clients.lock().unwrap().remove(&id);
    println!("Client disconnected: {}", id);
}