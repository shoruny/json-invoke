use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use axum_extra::TypedHeader;
use jsonrpc::{
    headers::signature::XSignature,
    math::Methods,
    rpc::{AsyncHandler, JsonRpcResponse, RpcJson, RpcRequest},
    ws::{handle_socket, AppState},
};
use parking_lot::RwLock;
use std::{
    collections::HashMap,
    sync::{Arc},
};
use tokio::net::TcpListener;

async fn handler(
    TypedHeader(XSignature(_sign)): TypedHeader<XSignature>,
    RpcJson(payload, id): RpcJson<RpcRequest<Methods>>,
) -> impl IntoResponse {
    let result = payload.execute().await;
    let resp = JsonRpcResponse::from_result(id, result);
    Json(resp)
}

pub async fn _ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket::<Methods>(socket, state, 0u32))
}

#[tokio::main]
async fn main() {
    let _state = AppState {
        lobby: Arc::new(RwLock::new(HashMap::new())),
        rooms: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/", get(|| async { "hello" }))
        .route("/ws", get(_ws_handler))
        .with_state(_state)
        .route("/http/rpc/math", post(handler));
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
