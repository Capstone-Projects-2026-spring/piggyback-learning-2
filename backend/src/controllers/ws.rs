use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, RwLock};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct WsMessage {
    pub sender: String,
    pub receiver: String,
    pub action: String,
    pub msg: String,
}

pub type Tx = mpsc::UnboundedSender<WsMessage>;

#[derive(Clone)]
pub struct AppState {
    pub users: Arc<RwLock<HashMap<String, Tx>>>,
}

#[derive(Deserialize, IntoParams)]
pub struct WsParams {
    /// Username to identify the WebSocket connection
    #[param(example = "john_doe")]
    pub username: String,
}

#[utoipa::path(
    get,
    path = "/ws",
    tag = "websocket",
    params(WsParams),
    responses(
        (status = 101, description = "WebSocket connection established", body = WsMessage),
        (status = 400, description = "Bad request"),
    )
)]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, params.username, state))
}
async fn handle_socket(stream: WebSocket, username: String, state: AppState) {
    let (mut sender, mut receiver) = stream.split();

    let (tx, mut rx) = mpsc::unbounded_channel::<WsMessage>();

    // register user
    state.users.write().await.insert(username.clone(), tx);

    // task: send outgoing messages
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if sender.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    // task: receive incoming messages
    let state_clone = state.clone();
    let uid = username.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            if let Ok(msg) = serde_json::from_str::<WsMessage>(&text) {
                let users = state_clone.users.read().await;

                if let Some(tx) = users.get(&msg.receiver) {
                    let _ = tx.send(msg.clone());
                }
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    state.users.write().await.remove(&uid);
}

pub fn routes() -> Routes {
    let app = AppState {
        users: Arc::new(RwLock::new(HashMap::new())),
    };

    Routes::new().add("/ws", get(ws_handler).with_state(app))
}
