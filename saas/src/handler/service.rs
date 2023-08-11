use std::{marker::PhantomData, fmt, convert::Infallible, task::{Context, Poll}};

use saas_core::{extract::Request, response::Response};
use tower_service::Service;

use crate::{
    BoxError,
    routing::IntoMakeService,
    extract::connect_info::IntoMakeServiceWithConnectInfo,
    body::{HttpBody, Bytes, Body},
};

use super::Handler;

pub struct HandlerService<H, T, S> {
    handler: H,
    state: S,
    _marker: PhantomData<fn() -> T>
}

impl<H, T, S> HandlerService<H, T, S> {
    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn into_make_service(self) -> IntoMakeService<HandlerService<H, T, S>> {
        IntoMakeService::new(self)
    }

    pub fn into_make_service_with_connect_info<C>(
        self
    ) -> IntoMakeServiceWithConnectInfo<HandlerService<H, T, S>, C> {
        IntoMakeServiceWithConnectInfo::new(self)
    }
}

impl<H, T, S> HandlerService<H, T, S> {
    pub fn new(handler: H, state: S) -> Self {
        Self {
            handler,
            state,
            _marker: PhantomData,
        }
    }
}

impl<H, T, S> fmt::Debug for HandlerService<H, T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HandlerService")
          .finish_non_exhaustive()
    }
}

impl<H, T, S> Clone for HandlerService<H, T, S>
where
    H: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            state: self.state.clone(),
            _marker: PhantomData,
        }
    }
}

impl<H, T, S, B> Service<Request<B>> for HandlerService<H, T, S>
where
    H: Handler<T, S> + Clone + Send + 'static,
    B: HttpBody<Data = Bytes> + Send + 'static,
    B::Error: Into<BoxError>,
    S: Clone + Send + Sync,
{
    type Error = Infallible;
    type Future = super::future::IntoServiceFuture<H::Future>;
    type Response = Response;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        use futures_util::future::FutureExt;

        let req = req.map(Body::new);
        let handler = self.handler.clone();
        let future = Handler::call(handler, req, self.state.clone());
        let future = future.map(Ok as _);
        super::future::IntoServiceFuture::new(future)
    }
}