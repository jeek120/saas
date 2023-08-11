use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use http::{Method};
use saas_core::extract::{FromRequest, Request};

use super::{
    has_content_type,
    rejection::{InvalidFormContentType, RawFormRejection},
};



#[derive(Debug)]
pub struct RawForm(pub Bytes);

#[async_trait]
impl<S> FromRequest<S> for RawForm
where
    S: Send + Sync,
{
    type Rejection = RawFormRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        if req.method() == Method::GET {
            let mut bytes = BytesMut::new();

            if let Some(query) = req.uri().query() {
                bytes.extend(query.as_bytes());
            }

            Ok(Self(bytes.freeze()))
        } else {
            if !has_content_type(req.headers(), &mime::APPLICATION_WWW_FORM_URLENCODED) {
                return Err(InvalidFormContentType.into());
            }

            Ok(Self(Bytes::from_request(req, state).await?))
        }
    }
}