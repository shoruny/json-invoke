use crate::rpc::{ AsyncHandler, JsonRpcResponse, RpcJson};
use axum::extract::ws::{Message, WebSocket};
use axum::response::Response;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::time::timeout;

#[derive(Deserialize)]
pub struct IncomingPacket<M> {
    #[serde(default, deserialize_with = "deserialize_optional_id")]
    pub id: Option<Value>,
    #[serde(flatten)] // 这里的 flatten 是灵魂
    pub method_call: M,
}
// 这是一个专门处理此逻辑的辅助函数
fn deserialize_optional_id<'de, D>(deserializer: D) -> Result<Option<Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // 使用 Option<Value> 接收，此时 null 会变成 Some(Value::Null)
    // 而字段缺失会因为 #[serde(default)] 调用此函数并返回 None
    Ok(Option::<Value>::deserialize(deserializer)
        .ok()
        .flatten()
        .or(Some(Value::Null)))
}

#[derive(Deserialize, Debug)]
struct AuthPacket {
    method: String,
    token: String,
}

struct S {
    name: String,
}
fn verify_token(token: String) -> Option<S> {
    Some(S { name: token })
}
async fn response_to_string(res: Response) -> Result<String, ()> {
    use axum::body::to_bytes;
    let bytes = to_bytes(res.into_body(), 1024 * 10).await.map_err(|_| ())?;
    String::from_utf8(bytes.to_vec()).map_err(|_| ())
}
pub async fn handle_socket<M>(socket: WebSocket, state: AppState, conn_id: u32)
where
    M: DeserializeOwned + AsyncHandler + Send + 'static {
    let (mut sender, mut receiver) = socket.split();

    let (tx, mut rx) = mpsc::channel::<String>(100);
    // --- 1. 等待身份验证消息 (限时 5 秒) ---
    let auth_timeout = Duration::from_secs(5);

    let user_info = match timeout(auth_timeout, receiver.next()).await {
        Ok(Some(Ok(Message::Text(text)))) => {
            // 尝试解析登录包，例如 {"method": "login", "token": "..."}
            if let Ok(auth_packet) = serde_json::from_str::<AuthPacket>(&text) {
                if let Some(user) = verify_token(auth_packet.token) && auth_packet.method == "login" {
                    // 验证通过，构造 UserInfo
                    Arc::new(UserInfo {
                        conn_id,
                        name: user.name,
                        login_at: 0u64,
                        tx: tx.clone(),
                        rooms: vec![],
                    })
                } else {
                    let _ = sender.send(Message::Text("Auth Failed".into())).await;
                    return; // 验证失败，直接断开
                }
            } else {
                let _ = sender.send(Message::Text("Auth Failed".into())).await;
                return; // 格式错误，断开
            }
        }
        _ => {
            let _ = sender.send(Message::Text("Auth Failed".into())).await;
            // 超时了、断开了或者发了非文本消息
            println!("连接 {} 认证超时或非法，强制断开", conn_id);
            return;
        }
    };
    {
        state.add(user_info);
        println!(
            "用户 {} 上线，当前在线人数: {}",
            conn_id,
            state.lobby.read().len()
        );
    }

    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(_e) = sender.send(Message::Text(msg.into())).await {
                break;
            }
        }
    });

    let tx_clone = tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                // 1. 尝试解析请求 (这里复用你的 Methods 枚举)
                // 假设我们定义了一个通用的 RpcRequest<T> 结构体
                match RpcJson::<IncomingPacket<M>>::from_str(&text) {
                    Ok(RpcJson(packet, id)) => {
                        let id = id.clone();
                        let need_response = id.is_some(); // 不含id，直接返回
                        let result = packet.method_call.execute().await; // 依然是 enum_dispatch！

                        // 3. 构造响应并转回 JSON 字符串
                        let resp = JsonRpcResponse::from_result(id, result);

                        if !need_response {
                            continue;
                        }

                        if let Ok(json_str) = serde_json::to_string(&resp) {
                            let _ = tx_clone.send(json_str.into()).await;
                        }
                    } ,Err(error_response) => {
                        // 解析失败，error_response 已经是按照 JSON-RPC 标准构造好的 Response
                        // 我们需要把 Response 里的 Body 转回字符串发给 WS 客户端
                        if let Ok(body_str) = response_to_string(error_response).await {
                            let _ = tx.send(body_str).await;
                        }
                    }
                }
            }
        }
    });

    // 5. 等待任意一个任务结束（连接断开）
    tokio::select! {
        _ = (&mut send_task) => {},
        _ = (&mut recv_task) => {},
    }

    // 6. 用户下线清理逻辑
    {
        state.leave(conn_id);
        println!(
            "用户 {} 下线，当前在线人数: {}",
            conn_id,
            state.lobby.read().len()
        );
    }
}

#[derive(Clone)]
pub struct UserInfo {
    pub conn_id: u32,
    pub name: String,
    pub login_at: u64,
    pub rooms: Vec<String>,
    pub tx: Sender<String>,
}

#[derive(Clone)]
pub struct AppState {
    pub lobby: Arc<RwLock<HashMap<u32, Arc<UserInfo>>>>,
    pub rooms: Arc<RwLock<HashMap<String, Vec<Arc<UserInfo>>>>>,
}

impl AppState {
    pub fn add(&self, user: Arc<UserInfo>) {
        let conn_id = user.conn_id;

        // 1. 插入到大厅，并拿回可能存在的“旧用户”
        let old_user = {
            let mut lobby = self.lobby.write();
            lobby.insert(conn_id, user) // insert 返回 Option<Arc<UserInfo>>
        };

        // 2. 如果存在旧用户，执行清理逻辑（踢人）
        if let Some(old_user) = old_user {
            let mut rooms = self.rooms.write();
            for room_id in &old_user.rooms {
                if let Some(room) = rooms.get_mut(room_id) {
                    // 清理该房间里的旧引用
                    room.retain(|u| u.conn_id != conn_id);
                }
            }
            // 清理可能产生的空房间
            rooms.retain(|_, users| !users.is_empty());

            // 此时 old_user 离开作用域，如果没有其他地方引用它，内存将被回收
            println!("用户 {} 已被新连接挤掉", conn_id);
        }
    }
    pub fn leave(&self, conn_id: u32) {
        let user = {
            let mut lobby = self.lobby.write();
            lobby.remove(&conn_id) // 直接 remove 会返回 Option<Arc<UserInfo>>
        };
        let Some(user) = user else { return };

        let mut rooms = self.rooms.write();
        for room_id in &user.rooms {
            if let Some(room) = rooms.get_mut(room_id) {
                room.retain(|u| !Arc::ptr_eq(u, &user));
            }
        }
        rooms.retain(|_name, users| !users.is_empty());
    }
}
