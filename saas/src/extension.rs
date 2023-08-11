use std::convert::Infallible;

use saas_core::{extract::{FromRequestParts, Request}, response::{IntoResponseParts, ResponseParts, IntoResponse, Response}};
use http::request::Parts;
use async_trait::async_trait;
use tower_layer::Layer;
use tower_service::Service;

use crate::extract::rejection::{ExtensionRejection, MissingExtension};

#[derive(Debug)]
#[must_use]
pub struct Extension<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for Extension<T>
where
    T: Clone + Send + Sync + 'static
{
    type Rejection = ExtensionRejection;

    async fn from_request_parts(parts: &mut Parts,state: &S) ->  Result<Self,Self::Rejection> {
        let value = parts
            .extensions
            .get::<T>()
            .ok_or_else(|| {
                MissingExtension::from_err(format!(
                    "Extension of type `{}` was not found. Perhaps you forgot to add it? See `saas::Extension`.",
                    std::any::type_name::<T>()
                ))
            })
            .map(|x| x.clone())?;
            
        Ok(Extension(value))
    }
}

saas_core::__impl_deref!(Extension);

impl<T> IntoResponseParts for Extension<T>
where
    T: Send + Sync + 'static
{
    type Error = Infallible;
    
    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        res.extensions_mut().insert(self.0);
        Ok(res)
    }
}

impl<T> IntoResponse for Extension<T>
where
    T: Send + Sync + 'static,
{
    fn into_response(self) -> Response {
        let mut res = ().into_response();
        res.extensions_mut().insert(self.0);
        res
    }
}

impl<S, T> Layer<S> for Extension<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Service = AddExtension<S, T>;

    fn layer(&self, inner: S) -> Self::Service {
        AddExtension {
            inner,
            value: self.0.clone(),
        }
    }
}


#[derive(Clone, Copy, Debug)]
pub struct AddExtension<S, T>{
    pub(crate) inner: S,
    pub(crate) value: T,
}

impl<ResBody, S, T> Service<Request<ResBody>> for AddExtension<S, T>
where
    S: Service<Request<ResBody>>,
    T: Clone + Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    #[inline]
    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ResBody>) -> Self::Future {
        req.extensions_mut().insert(self.value.clone());
        self.inner.call(req)
    }
}