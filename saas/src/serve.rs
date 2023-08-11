use std::{convert::Infallible, io, net::SocketAddr};

use crate::hyper1_tokio_io::TokioIo;
use saas_core::{body::Body, extract::Request, response::Response};
use futures_util::{future::poll_fn, FutureExt};
use hyper1::server::conn::http1;
use tokio::net::{TcpListener, TcpStream};
use tower_hyper_http_body_compat::{HttpBody04ToHttpBody1, HttpBody1ToHttpBody04};
use tower_service::Service;

#[cfg(feature = "tokio")]
pub async fn serve<M, S>(tcp_listener: TcpListener, mut make_service: M) -> io::Result<()>
where
    M: for<'a> Service<IncomingStream<'a>, Error = Infallible, Response = S>,
    S: Service<Request, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send,
{
    loop {
        let (tcp_stream, remote_addr) = tcp_listener.accept().await?;
        let tcp_stream = TokioIo::new(tcp_stream);

        // 查看是否准备好了
        poll_fn(|cx| make_service.poll_ready(cx))
            .await
            .unwrap_or_else(|err| match err {});
        
        // 将Service转为Router
        let service = make_service
            .call(IncomingStream {
                tcp_stream: &tcp_stream,
                remote_addr: remote_addr
            }).await
            .unwrap_or_else(|err| match err {});

        // F = fn(req: Request<Imcoming>) -> Future
        let service = hyper1::service::service_fn(move |req: Request<hyper1::body::Incoming>| {
            let mut service = service.clone();
            let req = req.map(|body| {
                let http_body_04 = HttpBody1ToHttpBody04::new(body);
                Body::new(http_body_04)
            });

            match poll_fn(|cx| service.poll_ready(cx)).now_or_never() {
                Some(Ok(())) => {},
                Some(Err(err)) => match err {},
                None => {
                    let mut res = Response::new(HttpBody04ToHttpBody1::new(Body::empty()));
                    *res.status_mut() = http::StatusCode::SERVICE_UNAVAILABLE;
                    return std::future::ready(Ok(res)).left_future();
                }
            }

            let future = service.call(req);
            async move {
                let response = future
                    .await
                    .unwrap_or_else(|err| match err {})
                    .map(HttpBody04ToHttpBody1::new);
                Ok::<_, Infallible>(response)
            }.right_future()
        });

        tokio::task::spawn(async move {
            match http1::Builder::new()
                .serve_connection(tcp_stream, service)
                .with_upgrades()
                .await
            {
                Ok(()) => {},
                Err(_err) => {

                }
            }
        });

    }
}

pub struct IncomingStream<'a> {
    tcp_stream: &'a TokioIo<TcpStream>,
    remote_addr: SocketAddr,
}

impl IncomingStream<'_> {
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.tcp_stream.inner().local_addr()
    }

    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }
}

