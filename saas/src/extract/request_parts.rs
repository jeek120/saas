use std::convert::Infallible;

use async_trait::async_trait;
use http::{request::Parts, Uri};
use saas_core::extract::FromRequestParts;

use crate::Extension;


#[cfg(feature = "original-uri")]
#[derive(Debug, Clone)]
pub struct OriginalUri(pub Uri);

#[cfg(feature = "original-uri")]
#[async_trait]
impl<S> FromRequestParts<S> for OriginalUri
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let uri = Extension::<Self>::from_request_parts(parts, state)
            .await
            .unwrap_or_else(|_| Extension(OriginalUri(parts.uri.clone())))
            .0;
        
        Ok(uri)
    }
}

#[cfg(feature = "original-uri")]
saas_core::__impl_deref!(OriginalUri: Uri);