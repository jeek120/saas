use saas_core::body::Body;
use http::{header, HeaderValue};


mod redirect;

#[cfg(feature = "tokio")]
pub mod sse;

#[doc(no_inline)]
#[cfg(feature = "json")]
pub use crate::Json;

#[cfg(feature = "form")]
#[doc(no_inline)]
pub use crate::form::Form;

#[doc(no_inline)]
pub use crate::Extension;


pub use saas_core::response:: {
    AppendHeaders, ErrorResponse, IntoResponse, IntoResponseParts, Response, ResponseParts, Result,
};

#[doc(inline)]
pub use self::redirect::Redirect;

#[doc(inline)]
#[cfg(feature = "tokio")]
pub use sse::Sse;

/// An HTML response.
///
/// Will automatically get `Content-Type: text/html`.
#[derive(Clone, Copy, Debug)]
#[must_use]
pub struct Html<T>(pub T);

impl<T> IntoResponse for Html<T>
where
    T: Into<Body>,
{
    fn into_response(self) -> Response {
        (
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static(mime::TEXT_HTML_UTF_8.as_ref()),
            )],
            self.0.into(),
        )
            .into_response()
    }
}

impl<T> From<T> for Html<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}
