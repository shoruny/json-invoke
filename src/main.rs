use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use axum_extra::TypedHeader;
use chat::{
    headers::signature::XSignature,
    math::Methods,
    ws::{handle_socket, AppState},
    AsyncHandler, JsonRpcResponse, RpcJson,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio::net::TcpListener;

async fn handler(
    TypedHeader(XSignature(sign)): TypedHeader<XSignature>,
    RpcJson(payload, id): RpcJson<Methods>,
) -> impl IntoResponse {
    println!("{}", sign);
    let result = payload.execute().await;
    let resp = JsonRpcResponse::from_result(id, result);
    Json(resp)
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket::<Methods>(socket, state, 0u32))
}

#[tokio::main]
async fn main() {
    let state = AppState {
        lobby: Arc::new(RwLock::new(HashMap::new())),
        rooms: Arc::new(RwLock::new(HashMap::new())),
    };
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state)
        .route("/http/rpc/math", post(handler));
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
