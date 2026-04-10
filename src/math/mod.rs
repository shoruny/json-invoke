use crate::{
    rpc::{AsyncHandler, RpcError},
    utils::decimal::ToDecimal,
};

use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
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
        Ok((self.a + self.b).as_json_number())
    }
}

#[async_trait]
impl AsyncHandler for MathAddArgs<SubArgs> {
    async fn execute(self) -> Result<Value, RpcError> {
        if self.a < 100f64 {
            Ok((self.a - self.b).as_json_number())
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
        Ok(res.as_f64().as_json_number())
    }
}
