use http::{StatusCode, HeaderValue, header};
use saas_core::response::{IntoResponse, Response};

#[must_use = "need to be returned from a handler or otherwise turned into a response to be usful"]
#[derive(Debug)]
pub struct Redirect {
    status_code: StatusCode,
    location: HeaderValue,
}

impl Redirect {
    pub fn to(uri: &str) -> Self {
        Self::with_status_code(StatusCode::SEE_OTHER, uri)
    }

    pub fn temporary(uri: &str) -> Self {
        Self::with_status_code(StatusCode::TEMPORARY_REDIRECT, uri)
    }

    pub fn permanent(uri: &str) -> Self {
        Self::with_status_code(StatusCode::PERMANENT_REDIRECT, uri)
    }

    fn with_status_code(status_code: StatusCode, uri: &str) -> Self {
        assert!(status_code.is_redirection(),
            "not a redirect status code: {}", status_code);
        Self {
            status_code,
            location: HeaderValue::try_from(uri).expect("URI isn't a valid header value"),
        }
    }
}

impl IntoResponse for Redirect {
    fn into_response(self) -> Response {
        (self.status_code, [(header::LOCATION, self.location)]).into_response()
    }
}