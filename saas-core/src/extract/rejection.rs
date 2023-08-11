use crate::__composite_rejection as composite_rejection;
use crate::__define_rejection as define_rejection;

use crate::{BoxError, Error};

composite_rejection! {
    pub enum FailedToBufferBody {
        LengthLimitError,
        UnknowBodyError,
    }
}

impl FailedToBufferBody {
    pub(crate) fn from_err<E>(err: E) -> Self
    where
        E: Into<BoxError>,
    {
        let box_error = match err.into().downcast::<Error>() {
            Ok(err) => err.into_inner(),
            Err(err) => err,
        };
        match box_error.downcast::<http_body::LengthLimitError>() {
            Ok(err) => Self::LengthLimitError(LengthLimitError::from_err(err)),
            Err(err) => Self::UnknowBodyError(UnknowBodyError::from_err(err)),
        }
    }
}

define_rejection! {
    #[status = PAYLOAD_TOO_LARGE]
    #[body = "Failed to buffer the request body"]
    pub struct LengthLimitError(Error);
}

define_rejection! {
    #[status = BAD_REQUEST]
    #[body = "Failed to buffer the request body"]
    pub struct UnknowBodyError(Error);
}

define_rejection! {
    #[status = BAD_REQUEST]
    #[body = "Request body didn't contain a valid UTF-8"]
    pub struct InvalidUtf8(Error);
}

composite_rejection! {
    pub enum BytesRejection {
        FailedToBufferBody,
    }
}

composite_rejection!{
    pub enum StringRejection {
        FailedToBufferBody,
        InvalidUtf8,
    }
}