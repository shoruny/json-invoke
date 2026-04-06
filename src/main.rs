use axum::{http::HeaderMap, response::IntoResponse, routing::post, Json, Router};
use chat::{math::Methods, AsyncHandler, JsonRpcResponse, RpcJson};
use tokio::net::TcpListener;

async fn handler(RpcJson(payload, id): RpcJson<Methods>) -> impl IntoResponse {
    let result = payload.execute().await;
    let resp = JsonRpcResponse::from_result(id, result);
    Json(resp)
}

// async fn ws_handler(
//     ws: WebSocketUpgrade,
//     headers: HeaderMap,
//     State(state): State<AppState>,
// ) -> impl IntoResponse {
//     let _ = headers;
//     ws.on_upgrade(move |socket| handle_socket(socket, state, 0u32))
// }
#[tokio::main]
async fn main() {
    // let state = AppState {
    //     online_users: Arc::new(RwLock::new(HashMap::new())),
    // };
    // let app = Router::new().route("/ws", any(handler)).with_state(state);
    let app = Router::new().route("/http/rpc/math", post(handler));
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
