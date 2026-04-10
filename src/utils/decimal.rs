use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

pub trait ToDecimal {
    fn as_decimal(self) -> Decimal;
}

impl ToDecimal for f64 {
    fn as_decimal(self) -> Decimal {
        Decimal::from_f64(self).unwrap_or_default()
    }
}
