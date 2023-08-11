use std::{convert::Infallible, fmt, task::{Context, Poll}, pin::Pin};
use crate::body::{Body, HttpBody};
use futures_util::Future;
use http::{HeaderMap, HeaderValue, header::{self, CONTENT_LENGTH}};
use pin_project_lite::pin_project;
use tower::{util::{Oneshot, BoxCloneService, MapResponseLayer}, ServiceExt, ServiceBuilder};
use tower_layer::Layer;
use tower_service::Service;
use crate::{response::Response, body::Bytes};
use saas_core::{extract::Request, response::IntoResponse};


pub struct Route<E = Infallible>(BoxCloneService<Request, Response, E>);
impl<E> Route<E> {
    pub(crate) fn new<T>(svc: T) -> Self
    where
        T: Service<Request, Error = E> + Clone + Send + 'static,
        T::Response: IntoResponse + 'static,
        T::Future: Send + 'static,
    {
        Self(BoxCloneService::new(
            svc.map_response(IntoResponse::into_response),
        ))
    }

    pub(crate) fn oneshot_inner(
        &mut self, 
        req: Request,
    ) -> Oneshot<BoxCloneService<Request, Response, E>, Request> {
        self.0.clone().oneshot(req)
    }

    pub(crate) fn layer<L, NewError>(self, layer: L) -> Route<NewError>
    where
        L: Layer<Route<E>> + Clone + Send + 'static,
        L::Service: Service<Request> + Clone + Send + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<NewError> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
        NewError: 'static,
    {
        let layer = ServiceBuilder::new()
            .map_request(|req: Request<_>| req.map(Body::new))
            .map_err(Into::into)
            .layer(MapResponseLayer::new(IntoResponse::into_response))
            .layer(layer)
            .into_inner();
        Route::new(layer.layer(self))
    }
}

impl<E> Clone for Route<E> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<E> fmt::Debug for Route<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Route").finish()
    }
}

impl<B, E> Service<Request<B>> for Route<E>
where
    B: HttpBody<Data = Bytes> + Send + 'static,
    B::Error: Into<saas_core::BoxError>,
{
    type Response = Response;
    type Error = E;
    type Future = RouteFuture<E>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let req = req.map(Body::new);
        RouteFuture::from_future(self.oneshot_inner(req))
    }
}

pin_project! {
    pub struct RouteFuture<E> {
        #[pin]
        kind: RouteFutureKind<E>,
        strip_body: bool,
        allow_header: Option<Bytes>,
    }
}

pin_project! {
    #[project = RouteFutureKindProj]
    enum RouteFutureKind<E> {
        Future {
            #[pin]
            future: Oneshot<
                BoxCloneService<Request, Response, E>,
                Request,
            >,
        },
        Response {
            response: Option<Response>,
        }
    }
}

impl<E> RouteFuture<E> {
    pub(crate) fn from_future(
        future: Oneshot<BoxCloneService<Request, Response, E>, Request>,
    ) -> Self {
        Self {
            kind: RouteFutureKind::Future { future },            
            strip_body: false,
            allow_header: None,
        }
    }

    pub(crate) fn strip_body(mut self, strip_body: bool) -> Self {
        self.strip_body = strip_body;
        self
    }

    pub(crate) fn allow_header(mut self, allow_header: Bytes) -> Self {
        self.allow_header = Some(allow_header);
        self
    }
}

impl<E> Future for RouteFuture<E> {
    type Output = Result<Response, E>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let mut res = match this.kind.project() {
            RouteFutureKindProj::Future { future } => match future.poll(cx) {
                Poll::Ready(Ok(res)) => res,
                Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
                Poll::Pending => return Poll::Pending,
            },
            RouteFutureKindProj::Response { response } => {
                response.take().expect("future polled after completion")
            }
        };

        set_allow_header(res.headers_mut(), this.allow_header);
        set_content_length(res.size_hint(), res.headers_mut());

        let res = if *this.strip_body {
            res.map(|_| Body::empty())
        } else {
            res
        };

        Poll::Ready(Ok(res))
    }
}

fn set_allow_header(headers: &mut HeaderMap, allow_header: &mut Option<Bytes>) {
    match allow_header.take() {
        Some(allow_header) if !headers.contains_key(header::ALLOW) => {
            headers.insert(
                header::ALLOW,
                HeaderValue::from_maybe_shared(allow_header).expect("invalid `Allow` header"),
            );
        }
        _ => {}
    }
}

fn set_content_length(size_hint: http_body::SizeHint, headers: &mut HeaderMap) {
    if headers.contains_key(CONTENT_LENGTH) {
        return;
    }

    if let Some(size) = size_hint.exact() {
        let header_value = if size == 0 {
            #[allow(clippy::declare_interior_mutable_const)]
            const ZERO: HeaderValue = HeaderValue::from_static("0");
            ZERO
        } else {
            let mut buffer = itoa::Buffer::new();
            HeaderValue::from_str(buffer.format(size)).unwrap()
        };
        headers.insert(header::CONTENT_LENGTH, header_value);
    }
}

pin_project! {
    pub struct InfallibleRouteFuture {
        #[pin]
        future: RouteFuture<Infallible>,
    }
}

impl InfallibleRouteFuture {
    pub(crate) fn new(future: RouteFuture<Infallible>) -> Self {
        Self { future }
    }
}

impl Future for InfallibleRouteFuture {
    type Output = Response;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match futures_util::ready!(self.project().future.poll(cx)) {
            Ok(res) => Poll::Ready(res),
            Err(err) => match err {},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traits() {
        use crate::test_helpers::*;
        assert_send::<Route<()>>();
    }
}