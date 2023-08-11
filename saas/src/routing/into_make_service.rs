use std::{convert::Infallible, task::Poll, future::ready};

use tower_service::Service;

#[derive(Debug, Clone)]
pub struct IntoMakeService<S> {
    svc: S,
}

impl<S> IntoMakeService<S> {
    pub(crate) fn new(s: S) -> Self {
        Self { svc: s}
    }
}

impl<S, T> Service<T> for IntoMakeService<S>
where
    S: Clone,
{
    type Response = S;
    type Error = Infallible;
    type Future = IntoMakeServiceFuture<S>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: T) -> Self::Future {
        IntoMakeServiceFuture::new(ready(Ok(self.svc.clone())))
    }
}

opaque_future! {
    pub type IntoMakeServiceFuture<S> = 
        std::future::Ready<Result<S, Infallible>>;
}