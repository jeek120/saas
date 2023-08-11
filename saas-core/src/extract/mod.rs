use crate::{body::Body, response::IntoResponse};
use async_trait::async_trait;
use http::request::Parts;
use std::convert::Infallible;

pub mod rejection;

mod default_body_limit;
mod from_ref;
mod request_parts;
mod tuple;

pub(crate) use self::default_body_limit::DefaultBodyLimitKind;
pub use self::{default_body_limit::DefaultBodyLimit, from_ref::FromRef};


pub type Request<T = Body> = http::Request<T>;

mod private {
    #[derive(Debug, Clone, Copy)]
    pub enum ViaParts {}

    #[derive(Debug, Clone, Copy)]
    pub enum ViaRequest {}
}

#[async_trait]
#[cfg_attr(
    nightly_error_messages,
    rustc_on_unimplemented(
        note = "Function argument is not a valid saas extractor."
    )
)]
pub trait FromRequestParts<S>: Sized {
    type Rejection: IntoResponse;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection>;
}

#[async_trait]
#[cfg_attr(
    nightly_error_messages,
    rustc_on_unimplemented(
        note = "Function argument is not a valid saas extractor."
    )
)]
pub trait FromRequest<S, M = private::ViaRequest>: Sized {
    type Rejection: IntoResponse;
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection>;
    
}

#[async_trait]
impl<S, T> FromRequest<S, private::ViaParts> for T
where
    S: Send + Sync,
    T: FromRequestParts<S>,
{
    type Rejection = <Self as FromRequestParts<S>>::Rejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let (mut parts, _) = req.into_parts();
        Self::from_request_parts(&mut parts, state).await
    }
}

#[async_trait]
impl<S, T> FromRequestParts<S> for Option<T>
where
    T: FromRequestParts<S>,
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<T>, Self::Rejection> {
        Ok(T::from_request_parts(parts, state).await.ok())
    }
}

#[async_trait]
impl<S, T> FromRequest<S> for Option<T>
where
    T: FromRequest<S>,
    S: Send + Sync,
{
    type Rejection = Infallible;
    async fn from_request(req: Request, state: &S) -> Result<Option<T>, Self::Rejection> {
        Ok(T::from_request(req, state).await.ok())
    }
}


#[async_trait]
impl<S, T> FromRequestParts<S> for Result<T, T::Rejection>
where
    T: FromRequestParts<S>,
    S: Send + Sync,
{
    type Rejection = Infallible;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(T::from_request_parts(parts, state).await)
    }
}

#[async_trait]
impl<S, T> FromRequest<S> for Result<T, T::Rejection>
where
    T: FromRequest<S>,
    S: Send + Sync,
{
    type Rejection = Infallible;
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        Ok(T::from_request(req, state).await)
    }
}