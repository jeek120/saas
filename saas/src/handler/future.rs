use std::{convert::Infallible, pin::Pin, task::{Context, Poll}};

use futures_util::{future::Map, Future};
use pin_project_lite::pin_project;
use saas_core::{response::Response, extract::Request};
use tower::util::Oneshot;
use tower_service::Service;

opaque_future!{
    // TODO: 不懂
    pub type IntoServiceFuture<F> = 
        Map<
            F,
            fn(Response) -> Result<Response, Infallible>
        >;
}

pin_project! {
    pub struct LayeredFuture<S>
    where
        S: Service<Request>
    {
        #[pin]
        inner: Map<Oneshot<S, Request>, fn(Result<S::Response, S::Error>) -> Response>,  
    }
}

impl<S> LayeredFuture<S>
where
    S: Service<Request>,
{
    pub(crate) fn new(
        inner: Map<Oneshot<S, Request>, fn(Result<S::Response, S::Error>) -> Response>,
    ) -> Self {
        Self { inner}
    }
}

impl<S> Future for LayeredFuture<S>
where
    S: Service<Request>
{
    type Output = Response;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().inner.poll(cx)
    }
}