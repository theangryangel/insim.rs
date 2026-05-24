use std::convert::Infallible;

use axum::{
    extract::{Form, FromRequest, Request},
    http::Method,
};
use serde::de::DeserializeOwned;
use validator::{Validate, ValidationErrors};

/// Deserialize an HTML form field that may be an empty string into `Option<T>`.
/// HTML forms always submit `""` for unselected/blank fields; this converts that to `None`.
pub fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    use serde::Deserialize as _;
    let s = Option::<String>::deserialize(de)?;
    match s.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => s.parse::<T>().map(Some).map_err(serde::de::Error::custom),
    }
}

pub struct Changeset<T> {
    pub params: T,
    errors: Option<ValidationErrors>,
}

impl<T: Default> Changeset<T> {
    pub fn empty() -> Self {
        Self {
            params: T::default(),
            errors: None,
        }
    }
}

impl<T> Changeset<T> {
    pub fn new(params: T) -> Self {
        Self {
            params,
            errors: None,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_none()
    }

    pub fn error(&self, field: &'static str) -> &str {
        let Some(ref errors) = self.errors else {
            return "";
        };
        errors
            .field_errors()
            .get(field)
            .and_then(|errs| errs.first())
            .and_then(|e| e.message.as_deref())
            .unwrap_or("")
    }
}

impl<T, S> FromRequest<S> for Changeset<T>
where
    T: DeserializeOwned + Default + Validate + Send,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        if req.method() == Method::GET {
            return Ok(Self::empty());
        }
        let data = match Form::<T>::from_request(req, state).await {
            Ok(Form(data)) => data,
            Err(_) => return Ok(Self::empty()),
        };
        let errors = data.validate().err();
        Ok(Self {
            params: data,
            errors,
        })
    }
}
