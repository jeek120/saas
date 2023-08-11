use super::{Extension, FromRequestParts};
use crate::{middleware::AddExtension, serve::IncomingStream};
use async_trait::async_trait;
use http::request::Parts;
use std::{
    convert::Infallible,
    fmt,
    future::ready,
    marker::PhantomData,
    net::SocketAddr,
    task::{Context, Poll},
};
use tower_layer::Layer;
use tower_service::Service;

pub struct IntoMakeServiceWithConnectInfo<S, C> {
    svc: S,
    _connect_info: PhantomData<fn() -> C>,
}

#[test]
fn traits() {
    use crate::test_helpers::*;
    assert_send::<IntoMakeServiceWithConnectInfo<(), NotSendSync>>();
}

impl<S, C> IntoMakeServiceWithConnectInfo<S, C> {
    pub(crate) fn new(svc: S) -> Self {
        Self {
            svc,
            _connect_info: PhantomData,
        }
    }
}

impl<S, C> fmt::Debug for IntoMakeServiceWithConnectInfo<S, C>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntoMakeServiceWithConnectInfo")
            .field("svc", &self.svc)
            .finish()
    }
}

impl<S, C> Clone for IntoMakeServiceWithConnectInfo<S, C>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            svc: self.svc.clone(),
            _connect_info: PhantomData,
        }
    }
}

pub trait Connected<T>: Clone + Send + Sync + 'static {
    fn connect_info(target: T) -> Self;
}

impl Connected<IncomingStream<'_>> for SocketAddr {
    fn connect_info(target: IncomingStream<'_>) -> Self {
        target.remote_addr()
    }
}

impl<S, C, T> Service<T> for IntoMakeServiceWithConnectInfo<S, C>
where
    S: Clone,
    C: Connected<T>,
{
    type Response = AddExtension<S, ConnectInfo<C>>;
    type Error = Infallible;
    type Future = ResponseFuture<S, C>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: T) -> Self::Future {
        let connect_info = ConnectInfo(C::connect_info(req));
        let svc = Extension(connect_info).layer(self.svc.clone());
        ResponseFuture::new(ready(Ok(svc)))
    } 
}

opaque_future! {
    pub type ResponseFuture<S, C> = 
        std::future::Ready<Result<AddExtension<S, ConnectInfo<C>>, Infallible>>;
}

#[derive(Clone, Copy, Debug)]
pub struct ConnectInfo<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for ConnectInfo<T>
where
    S: Send + Sync,
    T: Clone + Send + Sync + 'static,
{
    type Rejection = <Extension<Self> as FromRequestParts<S>>::Rejection;

    
    async fn from_request_parts(parts: &mut Parts,state: &S) ->  Result<Self,Self::Rejection> {
        match Extension::<Self>::from_request_parts(parts, state).await {
            Ok(Extension(connect_info)) => Ok(connect_info),
            Err(err) => match parts.extensions.get::<MockConnectInfo<T>>() {
                Some(MockConnectInfo(connect_info)) => Ok(Self(connect_info.clone())),
                None => Err(err),
            }
        }
    }
}

saas_core::__impl_deref!(ConnectInfo);

#[derive(Clone, Copy, Debug)]
pub struct MockConnectInfo<T>(pub T);

impl<S, T> Layer<S> for MockConnectInfo<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Service = <Extension<Self> as Layer<S>>::Service;

    fn layer(&self, inner: S) -> Self::Service {
        Extension(self.clone()).layer(inner)
    }
}
