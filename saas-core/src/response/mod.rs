use crate::body::Body;

mod append_headers;
mod into_response;
mod into_response_parts;

pub use self::{
    append_headers::AppendHeaders,
    into_response::IntoResponse,
    into_response_parts::{IntoResponseParts, ResponseParts, TryIntoHeaderError},
};

/// 类型[`http::Response`]别名，是saas绝大部分公用的Body
pub type Response<T = Body> = http::Response<T>;

pub type Result<T, E = ErrorResponse> = std::result::Result<T, E>;

impl<T> IntoResponse for Result<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Ok(ok) => ok.into_response(),
            Err(err) => err.0,
        }
    }
}

/// 一个 [`IntoResponse`] 的基础错误类型
/// 
/// 详细信息请看[`Result`]
#[derive(Debug)]
pub struct ErrorResponse(Response);