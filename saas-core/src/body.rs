//! Http body 工具类

use crate::{BoxError, Error};
use bytes::Bytes;
use bytes::{Buf, BufMut};
use futures_util::stream::Stream;
use futures_util::TryStream;
use http::HeaderMap;
use http_body::Body as _;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use sync_wrapper::SyncWrapper;

type BoxBody = http_body::combinators::UnsyncBoxBody<Bytes, Error>;

fn boxed<B>(body: B) -> BoxBody
where
    B: http_body::Body<Data = Bytes> + Send + 'static, 
    B::Error: Into<BoxError>,
{
    try_downcast(body).unwrap_or_else(|body| body.map_err(Error::new).boxed_unsync())
}

pub(crate) fn try_downcast<T, K>(k: K) -> Result<T, K>
where
    T: 'static,
    K: Send + 'static,
{
    let mut k = Some(k);
    if let Some(k) = <dyn std::any::Any>::downcast_mut::<Option<T>>(&mut k) {
        Ok(k.take().unwrap())
    } else {
        Err(k.unwrap())
    }
}

pub(crate) async fn to_bytes<T>(body: T) -> Result<Bytes, T::Error>
where
    T: http_body::Body,
{
    futures_util::pin_mut!(body);

    let mut first = if let Some(buf) = body.data().await {
        buf?
    } else {
        return Ok(Bytes::new());
    };

    let second = if let Some(buf) = body.data().await {
        buf?
    } else {
        return Ok(first.copy_to_bytes(first.remaining()));
    };

    let cap = first.remaining() + second.remaining() + body.size_hint().lower() as usize;
    let mut vec = Vec::with_capacity(cap);
    vec.put(first);
    vec.put(second);

    while let Some(buf) = body.data().await {
        vec.put(buf?);
    }

    Ok(vec.into())
}

/// body类型，用于Request和Response
#[derive(Debug)]
pub struct Body(BoxBody);

impl Body {
    /// 创建一个新的 `Body` that wrap another [`http_body::Body`]
    pub fn new<B>(body: B) -> Self 
    where
        B: http_body::Body<Data = Bytes> + Send + 'static,
        B::Error: Into<BoxError>,
    {
        try_downcast(body).unwrap_or_else(|body| Self(boxed(body)))
    }

    /// 创建一个空的Body
    pub fn empty() -> Self {
        Self::new(http_body::Empty::new())
    }

    /// 从Stream创建一个Body
    /// 
    /// [`Stream`]: futures::util::stream::Stream
    pub fn from_stream<S>(stream: S) -> Self 
    where
        S: TryStream + Send + 'static,
        S::Ok: Into<Bytes>,
        S::Error: Into<BoxError>,
    {
        Self::new(StreamBody{
            stream: SyncWrapper::new(stream),
        })
    }
}

impl Default for Body {
    fn default() -> Self {
        Self::empty()
    }
}

macro_rules! body_from_impl {
    ($ty:ty) => {
        impl From<$ty> for Body {
            fn from(buf: $ty) -> Self {
                Self::new(http_body::Full::from(buf))
            }
        }
    };
}

body_from_impl!(&'static [u8]);
body_from_impl!(std::borrow::Cow<'static, [u8]>);
body_from_impl!(Vec<u8>);

body_from_impl!(&'static str);
body_from_impl!(std::borrow::Cow<'static, str>);
body_from_impl!(String);

body_from_impl!(Bytes);

impl http_body::Body for Body {
    type Data = Bytes;
    type Error = Error;

    #[inline]
    fn poll_data(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
            Pin::new(&mut self.0).poll_data(cx)
    }

    #[inline]
    fn poll_trailers(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> std::task::Poll<Result<Option<HeaderMap>, Self::Error>> {
        Pin::new(&mut self.0).poll_trailers(cx)
    }
}

impl Stream for Body {
    type Item = Result<Bytes, Error>;

    #[inline]
    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll_data(cx)
    }
}

pin_project! {
    struct StreamBody<S> {
        #[pin]
        stream: SyncWrapper<S>,
    }
}

impl<S> http_body::Body for StreamBody<S>
where
S: TryStream,
S::Ok: Into<Bytes>,
S::Error: Into<BoxError>,
{
    type Data = Bytes;
    type Error = Error;

    fn poll_data(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let stream = self.project().stream.get_pin_mut();
        match futures_util::ready!(stream.try_poll_next(cx)) {
            Some(Ok(chunk)) => Poll::Ready(Some(Ok(chunk.into()))),
            Some(Err(err)) => Poll::Ready(Some(Err(Error::new(err)))),
            None => Poll::Ready(None),
        }
    }

    #[inline]
    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}


#[test]
fn test_try_downcast() {
    assert_eq!(try_downcast::<i32, _>(5_u32), Err(5_u32));
    assert_eq!(try_downcast::<i32, _>(5_i32), Ok(5_i32));
}