use crate::extract::Request;
use crate::extract::{rejection::*, FromRequest};
use async_trait::async_trait;
use saas_core::response::{IntoResponse, Response};
use bytes::{BufMut, Bytes, BytesMut};
use http::{
    header::{self, HeaderMap, HeaderValue},
    StatusCode,
};
use serde::{de::DeserializeOwned, Serialize};


#[derive(Debug, Clone, Copy, Default)]
pub struct Json<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for Json<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = JsonRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        if json_content_type(req.headers()) {
            let bytes = Bytes::from_request(req, state).await?;
            let deserializer = &mut serde_json::Deserializer::from_slice(&bytes);

            let value = match serde_path_to_error::deserialize(deserializer) {
                Ok(value) => value,
                Err(err) => {
                    let rejection = match err.inner().classify() {
                        serde_json::error::Category::Data => JsonDataError::from_err(err).into(),
                        serde_json::error::Category::Syntax | serde_json::error::Category::Eof => {
                            JsonSyntaxError::from_err(err).into()
                        }
                        serde_json::error::Category::Io => {
                            if cfg!(debug_assertions) {
                                unreachable!()
                            }else {
                                JsonSyntaxError::from_err(err).into()
                            }
                        }
                    };
                    return Err(rejection);
                }
            };

            Ok(Json(value))
        }else {
            Err(MissingJsonContentType.into())
        }
    }
}

fn json_content_type(headers: &HeaderMap) -> bool {
    let content_type = if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
        content_type
    }else {
        return false;
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    }else {
        return false;
    };

    let mime = if let Ok(mime) = content_type.parse::<mime::Mime>() {
        mime
    } else {
        return false;
    };

    let is_json_content_type = mime.type_() == "application"
        && (mime.subtype() == "json" || mime.suffix().map_or(false, |name| name == "json"));

    is_json_content_type
}

saas_core::__impl_deref!(Json);

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let mut buf = BytesMut::with_capacity(128).writer();

        match serde_json::to_writer(&mut buf, &self.0) {
            Ok(()) => (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                )],
                buf.into_inner().freeze(),
            ).into_response(),

            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                )],
                err.to_string(),
            ).into_response(),
        }
    }
}