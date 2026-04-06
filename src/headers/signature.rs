use axum::http::{HeaderName, HeaderValue};
use axum_extra::headers::{self, Header};

#[derive(Debug, Clone)]
pub struct XSignature(pub String);
impl Header for XSignature {
    fn name() -> &'static HeaderName {
        static NAME: HeaderName = HeaderName::from_static("x-signature");
        &NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(headers::Error::invalid)?;
        let s = value.to_str().map_err(|_| headers::Error::invalid())?;
        Ok(XSignature(s.to_string()))
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        let v = HeaderValue::from_str(&self.0).unwrap();
        values.extend(std::iter::once(v));
    }
}
