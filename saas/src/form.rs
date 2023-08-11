use async_trait::async_trait;
use http::{StatusCode, header::CONTENT_TYPE};
use saas_core::{extract::{FromRequest, Request}, RequestExt, response::IntoResponse};
use serde::{de::DeserializeOwned, Serialize};

use crate::extract::{rejection::{FormRejection, FailedToDeserializeForm, FailedToDeserializeFormBody, RawFormRejection}, RawForm};


#[cfg_attr(docsrs, doc(cfg(feature = "form")))]
#[derive(Debug, Clone, Copy, Default)]
#[must_use]
pub struct Form<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for Form<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = FormRejection;
    
    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let is_get_or_head = 
            req.method() == http::Method::GET || req.method() == http::Method::HEAD;

        match req.extract().await {
            Ok(RawForm(bytes)) => {
                let value = 
                    serde_urlencoded::from_bytes(&bytes).map_err(|err| -> FormRejection {
                        if is_get_or_head {
                            FailedToDeserializeForm::from_err(err).into()
                        }else {
                            FailedToDeserializeFormBody::from_err(err).into()
                        }
                    })?;

                Ok(Form(value))
            }
            Err(RawFormRejection::BytesRejection(r)) => Err(FormRejection::BytesRejection(r)),
            Err(RawFormRejection::InvalidFormContentType(r)) => {
                Err(FormRejection::InvalidFormContentType(r))
            }
        }
    }
}

impl<T> IntoResponse for Form<T>
where
    T: Serialize,
{
    fn into_response(self) -> saas_core::response::Response {
        match serde_urlencoded::to_string(&self.0) {
            Ok(body) => (
                [(CONTENT_TYPE, mime::APPLICATION_WWW_FORM_URLENCODED.as_ref())],
                body,
            ).into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        }
    }
}

saas_core::__impl_deref!(Form);