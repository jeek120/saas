use crate::response::{IntoResponse, Response};
use saas_core::extract::FromRequestParts;
use futures_util::future::BoxFuture;
use http::Request;
use std::{
    any::type_name,
    convert::Infallible,
    fmt,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use tower_layer::Layer;
use tower_service::Service;


pub fn map_response<F, T>(f: F) -> MapResponseLayer<F, (), T> {
    map_response_with_state((), f)
}

pub fn map_response_with_state<F, S, T>(state: S, f: F) -> MapResponseLayer<F, S, T> {
    MapResponseLayer {
        f,
        state,
        _extractor: PhantomData,
    }
}

#[must_use]
pub struct MapResponseLayer<F, S, T> {
    f: F,
    state: S,
    _extractor: PhantomData<fn() -> T>
}

impl<F, S, T> Clone for MapResponseLayer<F, S, T>
where
    F: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            f: self.f.clone(),
            state: self.state.clone(),
            _extractor: self._extractor,
        }
    }
}

impl<S, I, F, T> Layer<I> for MapResponseLayer<F, S, T>
where
    F: Clone,
    S: Clone,
{
    type Service = MapResponse<F, S, I, T>;

    fn layer(&self, inner: I) -> Self::Service {
        MapResponse {
            f: self.f.clone(),
            state: self.state.clone(),
            inner,
            _extractor: PhantomData,
        }
    }
}

impl<F, S, T> fmt::Debug for MapResponseLayer<F, S, T>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapResponseLayer")
            // Write out the type name, without quoting it as `&type_name::<F>()` would
            .field("f", &format_args!("{}", type_name::<F>()))
            .field("state", &self.state)
            .finish()
    }
}

pub struct MapResponse<F, S, I, T> {
    f: F,
    inner: I,
    state: S,
    _extractor: PhantomData<fn() -> T>,
}

impl<F, S, I, T> Clone for MapResponse<F, S, I, T>
where
    F: Clone,
    I: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            f: self.f.clone(),
            inner: self.inner.clone(),
            state: self.state.clone(),
            _extractor: self._extractor,
        }
    }
}

macro_rules! impl_service {
    (
        $($ty:ident),*
    ) => {
        #[allow(non_snake_case, unused_mut)]
        impl<F, Fut, S, I, B, ResBody, $($ty,)*> Service<Request<B>> for MapResponse<F, S, I, ($($ty,)*)>
        where
            F: FnMut($($ty,)* Response<ResBody>) -> Fut + Clone + Send + 'static,
            $($ty: FromRequestParts<S> + Send, )*
            Fut: Future + Send + 'static,
            Fut::Output: IntoResponse + Send + 'static,
            I: Service<Request<B>, Response = Response<ResBody>, Error = Infallible>
                + Clone
                + Send
                + 'static,
            I::Future: Send + 'static,
            B: Send + 'static,
            ResBody: Send + 'static,
            S: Clone + Send + Sync + 'static,
        {
            type Response = Response;
            type Error = Infallible;
            type Future = ResponseFuture;

            fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
                self.inner.poll_ready(cx)
            }

            fn call(&mut self, req: Request<B>) -> Self::Future {
                let not_ready_inner = self.inner.clone();
                let mut ready_inner = std::mem::replace(&mut self.inner, not_ready_inner);

                let mut f = self.f.clone();
                let _state = self.state.clone();

                let future = Box::pin(async move {
                    let (mut parts, body) = req.into_parts();

                    $(
                        let $ty = match $ty::from_request_parts(&mut parts, &_state).await {
                            Ok(value) => value,
                            Err(rejection) => return rejection.into_response(),
                        };
                    )*

                    let req = Request::from_parts(parts, body);

                    match ready_inner.call(req).await {
                        Ok(res) => {
                            f($($ty,)* res).await.into_response()
                        }
                        Err(err) => match err {}
                    }
                });

                ResponseFuture {
                    inner: future
                }
            }
        }
    };
}

impl_service!();
impl_service!(T1);
impl_service!(T1, T2);
impl_service!(T1, T2, T3);
impl_service!(T1, T2, T3, T4);
impl_service!(T1, T2, T3, T4, T5);
impl_service!(T1, T2, T3, T4, T5, T6);
impl_service!(T1, T2, T3, T4, T5, T6, T7);
impl_service!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_service!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_service!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_service!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_service!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_service!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_service!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_service!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_service!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);

impl<F, S, I, T> fmt::Debug for MapResponse<F, S, I, T>
where
    S: fmt::Debug,
    I: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapResponse")
            .field("f", &format_args!("{}", type_name::<F>()))
            .field("inner", &self.inner)
            .field("state", &self.state)
            .finish()
    }
}

pub struct ResponseFuture {
    inner: BoxFuture<'static, Response>,
}
impl Future for ResponseFuture {
    type Output = Result<Response, Infallible>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.inner.as_mut().poll(cx).map(Ok)
    }
}

impl fmt::Debug for ResponseFuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResponseFuture").finish()
    }
}
