use crate::backend::github_issues::sync_all_issues;
use crate::backend::{es, Config};
use axum::extract::{Extension, TypedHeader};
use axum::headers;
use axum::headers::{Error, Header, HeaderName};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use lazy_static::lazy_static;
use std::sync::Arc;

lazy_static! {
    pub static ref X_HUB_SIGNATURE: HeaderName = HeaderName::from_static("x-hub-signature");
}

pub async fn github_hook(
    TypedHeader(HubSignature(h)): TypedHeader<HubSignature>,
    Extension(conf): Extension<Arc<Config>>,
) -> impl IntoResponse {
    if h.is_empty() {
        return StatusCode::BAD_REQUEST;
    }
    let es_client = es::get_client().await.unwrap();
    match sync_all_issues(
        &conf.github_token,
        &conf.github_owner,
        &conf.github_repo,
        &es_client,
    )
    .await
    {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
// X-Hub-Signature
pub struct HubSignature(pub String);
impl Header for HubSignature {
    fn name() -> &'static HeaderName {
        &X_HUB_SIGNATURE
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, Error>
    where
        I: Iterator<Item = &'i headers::HeaderValue>,
    {
        values
            .next()
            .and_then(|v| v.to_str().ok().map(String::from))
            .map(HubSignature)
            .ok_or_else(Error::invalid)
    }

    fn encode<E: Extend<headers::HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(
            headers::HeaderValue::from_bytes(self.0.as_str().as_bytes()).unwrap(),
        ));
    }
}
