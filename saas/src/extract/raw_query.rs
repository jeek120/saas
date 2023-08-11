use std::convert::Infallible;

use async_trait::async_trait;
use http::request::Parts;
use saas_core::extract::FromRequestParts;


pub struct RawQuery(pub Option<String>);

#[async_trait]
impl<S> FromRequestParts<S> for RawQuery
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let query = parts.uri.query().map(|query| query.to_owned());
        Ok(Self(query))
    }

}