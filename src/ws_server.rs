use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{broadcast, Mutex};
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct WebSocketServer {
    tx: broadcast::Sender<String>,
    client_messages: Arc<Mutex<Vec<ClientMessage>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "send_message")]
    SendMessage { content: String },
}

impl WebSocketServer {
    pub async fn new(port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, _rx) = broadcast::channel(100);
        let client_messages = Arc::new(Mutex::new(Vec::new()));

        let server = WebSocketServer {
            tx: tx.clone(),
            client_messages: client_messages.clone(),
        };

        let app_state = AppState {
            tx,
            client_messages,
        };

        let app = Router::new()
            .route("/", get(serve_index))
            .route("/ws", get(ws_handler))
            .with_state(app_state);

        let addr = format!("0.0.0.0:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        Ok(server)
    }

    pub async fn broadcast_json(&self, value: &serde_json::Value) {
        let _ = self.tx.send(serde_json::to_string(value).unwrap());
    }

    pub async fn receive_message(&self) -> Option<ClientMessage> {
        let mut messages = self.client_messages.lock().await;
        if !messages.is_empty() {
            Some(messages.remove(0))
        } else {
            None
        }
    }
}

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
    client_messages: Arc<Mutex<Vec<ClientMessage>>>,
}

async fn serve_index() -> impl IntoResponse {
    Html(include_str!("../web/index.html"))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    // Send initial connection message
    let _ = sender.send(Message::Text(
        serde_json::json!({
            "type": "connected",
            "message": "Connected to Chat-IBM"
        }).to_string()
    )).await;

    // Spawn task to send messages to client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Spawn task to receive messages from client
    let client_messages = state.client_messages.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            use futures_util::stream::StreamExt;

            if let Ok(msg) = serde_json::from_str::<ClientMessage>(&text) {
                let mut messages = client_messages.lock().await;
                messages.push(msg);
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
}