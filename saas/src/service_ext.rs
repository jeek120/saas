#[cfg(feature = "tokio")]
use crate::extract::connect_info::IntoMakeServiceWithConnectInfo;
use crate::routing::IntoMakeService;
use tower_service::Service;

pub trait ServiceExt<R>: Service<R> + Sized {
    fn into_make_service(self) -> IntoMakeService<Self>;

    #[cfg(feature = "tokio")]
    fn into_make_service_with_connect_info<C>(self) -> IntoMakeServiceWithConnectInfo<Self, C>;
}

impl<S, R> ServiceExt<R> for S
where
    S: Service<R> + Sized,
{
    fn into_make_service(self) -> IntoMakeService<Self> {
        IntoMakeService::new(self)
    }

    #[cfg(feature = "tokio")]
    fn into_make_service_with_connect_info<C>(self) -> IntoMakeServiceWithConnectInfo<Self, C> {
        IntoMakeServiceWithConnectInfo::new(self)
    }
}