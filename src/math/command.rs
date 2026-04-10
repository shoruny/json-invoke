use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

use crate::rpc::{to_json_num, AsyncHandler, RpcError};

#[derive(Deserialize)]
#[serde(tag = "method", content = "params")]
#[serde(rename_all = "snake_case")]
pub enum Command {
    #[serde(rename = "math:add")]
    #[serde(alias = "add")]
    Add {
        #[serde(alias = "p1")]
        a: f64,
        b: f64,
    },
    Add2 {
        a: f64,
        b: f64,
        c: f64,
    },
    Sub {
        a: f64,
        b: f64,
    },
    Mul {
        a: f64,
        b: f64,
    },
    Div {
        a: f64,
        b: f64,
    },
}

#[async_trait]
impl AsyncHandler for Command {
    async fn execute(self) -> Result<Value, RpcError> {
        match self {
            Self::Add { a, b } => Ok(to_json_num(a + b)),
            Self::Add2 { a, b, c } => Ok(to_json_num(a + b + c)),
            Self::Sub { a, b } => Ok(to_json_num(a - b)),
            Self::Mul { a, b } => Ok(to_json_num(a * b)),
            Self::Div { a, b } => {
                if b == 0.0 {
                    return Err(RpcError::error(500, "div by 0".into()));
                }
                Ok(to_json_num(a / b))
            }
        }
    }
}
