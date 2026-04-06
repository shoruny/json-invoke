use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::Value;

use crate::RpcError;
use crate::{to_json_num, AsyncHandler};

#[derive(Deserialize)]
#[serde(tag = "method", content = "params", rename_all = "snake_case")] // 关键：JSON 结构映射
#[enum_dispatch(AsyncHandler)] // 关键点：告诉枚举去分发这个 Trait
pub enum Methods {
    Add(AddArgs),
    Sub(SubArgs),
    Mul(MulArgs),
}

#[derive(Deserialize)]
pub struct AddArgs {
    pub a: f64,
    pub b: f64,
}

#[async_trait]
impl AsyncHandler for AddArgs {
    async fn execute(self) -> Result<Value, RpcError> {
        Ok(to_json_num(self.a + self.b))
    }
}

#[derive(Deserialize)]
pub struct SubArgs {
    pub a: f64,
    pub b: f64,
}

#[async_trait]
impl AsyncHandler for SubArgs {
    async fn execute(self) -> Result<Value, RpcError> {
        if self.a < 100f64 {
            Ok(to_json_num(self.a - self.b))
        } else {
            Err(RpcError::error(500, "eee".into()))
        }
    }
}

#[derive(Deserialize)]
pub struct MulArgs {
    pub a: f64,
    pub b: f64,
}

#[async_trait]
impl AsyncHandler for MulArgs {
    async fn execute(self) -> Result<Value, RpcError> {
        let a_dec = Decimal::from_f64(self.a).unwrap_or_default();
        let b_dec = Decimal::from_f64(self.b).unwrap_or_default();
        let res = a_dec * b_dec;
        // *dec!(self.b);
        Ok(to_json_num(res.to_f64().unwrap_or_default()))
    }
}
