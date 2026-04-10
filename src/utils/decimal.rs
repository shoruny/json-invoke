use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use serde_json::{json, Value};
pub trait ToF64 {
    fn as_f64(self) -> f64;
}
impl ToF64 for Decimal {
    fn as_f64(self) -> f64 {
        self.to_f64().unwrap_or_default()
    }
}
pub trait ToDecimal {
    fn as_decimal(self) -> Decimal;
    fn as_json_number(self) -> Value;
}

impl ToDecimal for f64 {
    fn as_decimal(self) -> Decimal {
        Decimal::from_f64(self).unwrap_or_default()
    }

    fn as_json_number(self) -> Value {
        if self == self.trunc() {
            json!(self as i64)
        } else {
            json!(self)
        }
    }
}
