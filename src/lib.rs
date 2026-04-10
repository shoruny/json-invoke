pub mod headers;
pub mod math;
pub mod rpc;
pub mod ws;
// // http 提取器
// impl<S, T> FromRequest<S> for RpcJson<T>
// where
//     T: serde::de::DeserializeOwned,
//     S: Send + Sync,
// {
//     type Rejection = Response;

//     async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
//         // 1. 先调用原生的 Json<Value> 提取器，把整个包读进来
//         let Json(full_value) = Json::<Value>::from_request(req, state).await.map_err(|_| {
//             // 如果连 JSON 都解析不了（Parse Error）
//             jsonrpc_error(None, -32700, "Parse error")
//         })?;

//         // 2. 提取 id（不管后面成不成功，先把 id 拿到手）
//         let id = match full_value.get("id").cloned() {
//             Some(e) => Some(e),
//             None => None,
//         };

//         // 3. 尝试将整个 Value 转换为你的 Methods 枚举 (T)
//         // serde_json::from_value 不会消耗数据，非常安全
//         match serde_json::from_value::<T>(full_value) {
//             Ok(payload) => Ok(RpcJson(payload, id)),
//             Err(e) => {
//                 let err_msg = e.to_string();
//                 let code = if err_msg.contains("unknown variant") {
//                     -32601 // Method not found
//                 } else {
//                     -32602 // Invalid params
//                 };
//                 Err(jsonrpc_error(id, code, &err_msg))
//             }
//         }
//     }
// }
