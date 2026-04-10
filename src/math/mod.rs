use crate::{
    rpc::{to_json_num, AsyncHandler, RpcError},
    utils::decimal::ToDec,
};
use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Debug, Serialize)]
pub struct MathAddArgs<Op> {
    pub a: f64,
    pub b: f64,
    #[serde(skip)]
    _op: std::marker::PhantomData<Op>,
}

#[derive(Deserialize, Debug)]
pub struct AddArgs;
#[derive(Deserialize, Debug)]
pub struct SubArgs;
#[derive(Deserialize, Debug)]
pub struct MulArgs;

#[derive(Deserialize, Debug, Serialize)]
#[serde(tag = "method", content = "params", rename_all = "snake_case")] // 关键：JSON 结构映射
#[enum_dispatch(AsyncHandler)] // 关键点：告诉枚举去分发这个 Trait
pub enum Methods {
    #[serde(rename = "math:add")]
    Add(MathAddArgs<AddArgs>),
    Sub(MathAddArgs<SubArgs>),
    Mul(MathAddArgs<MulArgs>),
}

#[async_trait]
impl AsyncHandler for MathAddArgs<AddArgs> {
    async fn execute(self) -> Result<Value, RpcError> {
        Ok(to_json_num(self.a + self.b))
    }
}

#[async_trait]
impl AsyncHandler for MathAddArgs<SubArgs> {
    async fn execute(self) -> Result<Value, RpcError> {
        if self.a < 100f64 {
            Ok(to_json_num(self.a - self.b))
        } else {
            Err(RpcError::error(500, "eee".into()))
        }
    }
}

#[async_trait]
impl AsyncHandler for MathAddArgs<MulArgs> {
    async fn execute(self) -> Result<Value, RpcError> {
        let res = self.a.as_decimal() * self.b.as_decimal();
        // *dec!(self.b);
        Ok(to_json_num(res.to_f64().unwrap_or_default()))
    }
}
