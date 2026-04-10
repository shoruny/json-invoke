use async_trait::async_trait;
use axum::{
    extract::FromRequest,
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
// use enum_dispatch::enum_dispatch;
// use math::{AddArgs, MathArgs, Methods, MulArgs, SubArgs};
use serde::Serialize;
use serde_json::{json, Value};

#[async_trait]
// #[enum_dispatch]
pub trait AsyncHandler {
    // 假设每个处理逻辑都需要访问 State 和 用户 ID
    async fn execute(self) -> Result<Value, RpcError>;
}

pub fn to_json_num(n: f64) -> serde_json::Value {
    if n == n.trunc() {
        json!(n as i64)
    } else {
        json!(n)
    }
}

#[derive(Debug)]
pub enum RpcError {
    ParseError,                 // -32700
    InvalidRequest,             // -32600
    MethodNotFound,             // -32601
    InvalidParams(String),      // -32602
    InternalError,              // -32603
    BusinessError(i32, String), // 自定义业务错误
}

impl RpcError {
    pub fn error(code: i32, message: String) -> RpcError {
        RpcError::BusinessError(code, message)
    }

    // 转换为符合规范的 (code, message)
    pub fn code_msg(&self) -> (i32, String) {
        match self {
            RpcError::ParseError => (-32700, "Parse error".into()),
            RpcError::InvalidRequest => (-32600, "Invalid Request".into()),
            RpcError::MethodNotFound => (-32601, "Method not found".into()),
            RpcError::InvalidParams(m) => (-32602, format!("Invalid params: {}", m)),
            RpcError::InternalError => (-32603, "Internal error".into()),
            RpcError::BusinessError(c, m) => (*c, m.clone()),
        }
    }
}

#[derive(Serialize)]
pub struct JsonRpcResponse {
    #[serde(skip)]
    pub jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>, // 注意：如果请求有 ID，响应必须返回 ID（即使是 null）
}

impl JsonRpcResponse {
    pub fn from_result(id: Option<Value>, res: Result<Value, RpcError>) -> Self {
        match res {
            Ok(val) => Self::success(id, Some(val)),
            Err(err) => {
                let (code, message) = err.code_msg();
                Self::error(
                    id,
                    Some(json!({
                        "code":  code,
                        "message": message
                    })),
                )
            }
        }
    }

    pub fn error(id: Option<Value>, error: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0", // 内部自动填好
            error,
            id,
            result: None,
        }
    }
    pub fn success(id: Option<Value>, result: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0", // 内部自动填好
            result,
            id,
            error: None,
        }
    }
}
// 1. 定义一个统一的 JSON-RPC 错误响应结构
fn jsonrpc_error(id: Option<serde_json::Value>, code: i32, message: &str) -> Response {
    let body = JsonRpcResponse::error(
        id,
        Some(json!({
            "code":  code,
            "message": message
        })),
    );
    (StatusCode::OK, Json(body)).into_response() // 注意：RPC 规范通常返回 200 OK，内部带错误码
}

// websocket提取器
pub struct RpcJson<T>(pub T, pub Option<Value>);
impl<T> RpcJson<T>
where
    T: serde::de::DeserializeOwned,
{
    /// 专门为 WebSocket 设计的解析函数
    pub fn from_str(text: &str) -> Result<Self, Response> {
        // 1. 基础 JSON 解析 (Parse error)
        let full_value: Value =
            serde_json::from_str(text).map_err(|_| jsonrpc_error(None, -32700, "Parse error"))?;

        // 2. 提取 ID
        let id = full_value.get("id").cloned();

        // 3. 业务解析 (Method not found / Invalid params)
        match serde_json::from_value::<T>(full_value) {
            Ok(payload) => Ok(RpcJson(payload, id)),
            Err(e) => {
                let err_msg = e.to_string();
                let code = if err_msg.contains("unknown variant") {
                    -32601 // Method not found (找不到这个冒号分隔的方法名)
                } else {
                    -32602 // Invalid params (参数对不上)
                };
                Err(jsonrpc_error(id, code, &err_msg))
            }
        }
    }
}

// http 提取器
impl<S, T> FromRequest<S> for RpcJson<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // 1. 先调用原生的 Json<Value> 提取器，把整个包读进来
        let Json(full_value) = Json::<Value>::from_request(req, state).await.map_err(|_| {
            // 如果连 JSON 都解析不了（Parse Error）
            jsonrpc_error(None, -32700, "Parse error")
        })?;

        // 2. 提取 id（不管后面成不成功，先把 id 拿到手）
        let id = match full_value.get("id").cloned() {
            Some(e) => Some(e),
            None => None,
        };

        // 3. 尝试将整个 Value 转换为你的 Methods 枚举 (T)
        // serde_json::from_value 不会消耗数据，非常安全
        match serde_json::from_value::<T>(full_value) {
            Ok(payload) => Ok(RpcJson(payload, id)),
            Err(e) => {
                let err_msg = e.to_string();
                let code = if err_msg.contains("unknown variant") {
                    -32601 // Method not found
                } else {
                    -32602 // Invalid params
                };
                Err(jsonrpc_error(id, code, &err_msg))
            }
        }
    }
}
