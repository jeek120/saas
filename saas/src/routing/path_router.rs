use std::{collections::HashMap, sync::Arc, borrow::Cow, convert::Infallible, fmt};
use crate::{
        response::{IntoResponse}
    };
use matchit::MatchError;
use saas_core::extract::Request;
use serde::de::IntoDeserializer;
use tower_layer::Layer;
use tower_service::Service;

use super::{
        FALLBACK_PARAM_PATH,
        NEST_TAIL_PARAM,
        Route,
        Endpoint, method_routing::MethodRouter, route::RouteFuture, url_params, not_found::NotFound, strip_prefix::StripPrefix, RouteId,
    };


pub(super) struct PathRouter<S, const IS_FALLBACK: bool> {
    routes: HashMap<RouteId, Endpoint<S>>,
    node: Arc<Node>,
    prev_route_id: RouteId,
}

impl<S> PathRouter<S, true>
where
    S: Clone + Send + Sync + 'static,
{
    pub(super) fn new_fallback() -> Self {
        let mut this = Self::default();
        this.set_fallback(Endpoint::Route(Route::new(NotFound)));
        this
    }

    pub(super) fn set_fallback(&mut self, endpoint: Endpoint<S>) {
        self.replace_endpoint("/", endpoint.clone());
        self.replace_endpoint(FALLBACK_PARAM_PATH, endpoint);
    }
}

impl<S, const IS_FALLBACK:bool> PathRouter<S, IS_FALLBACK>
where
    S: Clone + Send + Sync + 'static,
{
    pub(super) fn route(
        &mut self,
        path: &str,
        method_router: MethodRouter<S>,
    ) -> Result<(), Cow<'static, str>> {
        fn validate_path(path: &str) -> Result<(), &'static str> {
            if path.is_empty() {
                return Err("Paths must start with a `/`. Use \"/\" for root routes");
            } else if !path.starts_with('/') {
                return Err("Paths must start with a `/`");
            }

            Ok(())
        }

        validate_path(path)?;

        let id = self.next_route_id();

        let endpoint = if let Some((route_id, Endpoint::MethodRouter(prev_method_router))) = self
            .node
            .path_to_route_id
            .get(path)
            .and_then(|route_id| self.routes.get(route_id).map(|svc| (*route_id, svc)))
        {
            let service = Endpoint::MethodRouter(
                prev_method_router
                    .clone()
                    .merge_for_path(Some(path), method_router),
            );
            self.routes.insert(route_id, service);
            return Ok(());
        } else {
            Endpoint::MethodRouter(method_router)
        };

        self.set_node(path, id)?;
        self.routes.insert(id, endpoint);

        Ok(())
    }

    pub(super) fn route_service<T>(
        &mut self,
        path: &str,
        service: T,
    ) -> Result<(), Cow<'static, str>>
    where
        T: Service<Request, Error = Infallible> + Clone + Send + 'static,
        T::Response: IntoResponse,
        T::Future: Send + 'static,
    {
        self.route_endpoint(path, Endpoint::Route(Route::new(service)))
    }

    pub(super) fn route_endpoint(
        &mut self,
        path: &str,
        endpoint: Endpoint<S>,
    ) -> Result<(), Cow<'static, str>> {
        if path.is_empty() {
            return Err("Paths must start with a `/`. Use \"/\" for root routes".into());
        } else if !path.starts_with('/') {
            return Err("Paths must start with a `/`".into());
        }

        let id = self.next_route_id();
        self.set_node(path, id)?;
        self.routes.insert(id, endpoint);

        Ok(())
    }

    fn set_node(&mut self, path: &str, id: RouteId) -> Result<(), String> {
        let mut node =
            Arc::try_unwrap(Arc::clone(&self.node)).unwrap_or_else(|node| (*node).clone());
        
        if let Err(err) = node.insert(path, id) {
            return Err(format!("Invalid route {path:?}: {err}"));
        }
        self.node = Arc::new(node);
        Ok(())
    }

    pub(super) fn merge(
        &mut self,
        other: PathRouter<S, IS_FALLBACK>,
    ) -> Result<(), Cow<'static, str>> {
        let PathRouter {
            routes,
            node,
            prev_route_id: _,
        } = other;

        for (id, route) in routes {
            let path = node
                .route_id_to_path
                .get(&id)
                .expect("no path for route id. This is a bug inx saas. Please file an issue");
            
            if IS_FALLBACK && (&**path == "/" || &**path == FALLBACK_PARAM_PATH) {
                self.replace_endpoint(path, route);
            }else {
                match route {
                    Endpoint::MethodRouter(method_router) => self.route(path, method_router)?,
                    Endpoint::Route(route) => self.route_service(path, route)?,
                }
            }
        }
        Ok(())
    }

    pub(super) fn nest(
        &mut self,
        path: &str,
        router: PathRouter<S, IS_FALLBACK>,
    ) -> Result<(), Cow<'static, str>> {
        let prefix = validate_nest_path(path);

        let PathRouter {
            routes,
            node,
            prev_route_id: _,
        } = router;

        for (id, endpoint) in routes {
            let inner_path = node
                .route_id_to_path
                .get(&id)
                .expect("no path for route id. This is a bug in saas. Please filie an issue");

            let path = path_for_nested_route(prefix, inner_path);

            match endpoint.layer(StripPrefix::layer(prefix)) {
                Endpoint::MethodRouter(method_router) => {
                    self.route(&path, method_router)?;
                }
                Endpoint::Route(route) => {
                    self.route_endpoint(&path, Endpoint::Route(route))?;
                }
            }
        }
        Ok(())
    }

    pub(super) fn nest_service<T>(&mut self, path: &str, svc: T) -> Result<(), Cow<'static, str>>
    where
        T: Service<Request, Error = Infallible> + Clone + Send + 'static,
        T::Response: IntoResponse,
        T::Future: Send + 'static,
    {
        let path = validate_nest_path(path);
        let prefix = path;

        let path = if path.ends_with('/') {
            format!("{path}*{NEST_TAIL_PARAM}")
        } else {
            format!("{path}/*{NEST_TAIL_PARAM}")
        };

        let endpoint = Endpoint::Route(Route::new(StripPrefix::new(svc, prefix)));

        self.route_endpoint(&path, endpoint.clone())?;

        self.route_endpoint(prefix, endpoint.clone())?;

        if !prefix.ends_with('/') {
            self.route_endpoint(&format!("{prefix}/"), endpoint)?;
        }

        Ok(())
    }

    pub(super) fn layer<L>(self, layer: L) -> PathRouter<S, IS_FALLBACK>
    where
        L: Layer<Route> + Clone + Send + 'static,
        L::Service: Service<Request> + Clone + Send + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        let routes = self
            .routes
            .into_iter()
            .map(|(id, endpoint)| {
                let route = endpoint.layer(layer.clone());
                (id, route)
            })
            .collect();

        PathRouter {
            routes,
            node: self.node,
            prev_route_id: self.prev_route_id,
        }
    }

    #[track_caller]
    pub(super) fn route_layer<L>(self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + 'static,
        L::Service: Service<Request> + Clone + Send + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        if self.routes.is_empty() {
            panic!(
                "Adding a route_layer before any routes is a no-op. \
                Add the routes you want the layer to apply to first."
            );
        }

        let routes = self
            .routes
            .into_iter()
            .map(|(id, endpoint)| {
                let route = endpoint.layer(layer.clone());
                (id, route)
            })
            .collect();

        PathRouter {
            routes,
            node: self.node,
            prev_route_id: self.prev_route_id,
        }
    }

    pub(super) fn with_state<S2>(self, state: S) -> PathRouter<S2, IS_FALLBACK> {
        let routes = self
            .routes
            .into_iter()
            .map(|(id, endpoint)| {
                let endpoint: Endpoint<S2> = match endpoint {
                    Endpoint::MethodRouter(method_router) => {
                        Endpoint::MethodRouter(method_router.with_state(state.clone()))
                    }
                    Endpoint::Route(route) => Endpoint::Route(route),
                };
                (id, endpoint)
            })
            .collect();

        PathRouter {
            routes,
            node: self.node,
            prev_route_id: self.prev_route_id,
        }
    }

    pub(super) fn call_with_state(
        &mut self,
        mut req: Request,
        state: S,
    ) -> Result<RouteFuture<Infallible>, (Request, S)> {
        #[cfg(feature = "original-uri")]
        {
            use crate::extract::OriginalUri;

            if req.extensions().get::<OriginalUri>().is_none() {
                let original_uri = OriginalUri(req.uri().clone());
                req.extensions_mut().insert(original_uri);
            }
        }

        let path = req.uri().path().to_owned();

        match self.node.at(&path) {
            Ok(match_) => {
                let id = *match_.value;

                if !IS_FALLBACK {
                    #[cfg(feature = "matched-path")]
                    crate::extract::matched_path::set_matched_path_for_request(
                        id,
                        &self.node.route_id_to_path,
                        req.extensions_mut(),
                    );
                }

                // url参数中，将路径中的参数加进去
                url_params::insert_url_params(req.extensions_mut(), match_.params);

                // 根据路由id查找终端
                let endpoint = self
                    .routes
                    .get_mut(&id)
                    .expect("no route for id. This is a bug in saas. Please file an issue");

                // 根据终端类型调用方法
                match endpoint {
                    Endpoint::MethodRouter(method_router) => {
                        Ok(method_router.call_with_state(req, state))
                    }
                    Endpoint::Route(route) => Ok(route.clone().call(req)),
                }
            }

            Err(
                MatchError::NotFound
                | MatchError::ExtraTrailingSlash
                | MatchError::MissingTrailingSlash,
            ) => Err((req, state)),
        }
    }

    pub(super) fn replace_endpoint(&mut self, path: &str, endpoint: Endpoint<S>) {
        match self.node.at(path) {
            Ok(match_) => {
                let id = *match_.value;
                self.routes.insert(id, endpoint);
            }
            Err(_) => self
                .route_endpoint(path, endpoint)
                .expect("path wasn't matched so endpoint shouldn't exist"),
        }
    }

    fn next_route_id(&mut self) -> RouteId {
        let next_id = self
            .prev_route_id
            .0
            .checked_add(1)
            .expect("Over `u32::MAX` routes crated. If you need this, please file an issue");

        self.prev_route_id = RouteId(next_id);
        self.prev_route_id
    }
}

impl<S, const IS_FALLBACK: bool> Default for PathRouter<S, IS_FALLBACK> {
    fn default() -> Self {
        Self {
            routes: Default::default(),
            node: Default::default(),
            prev_route_id: RouteId(0),
        }
    }
}

impl<S, const IS_FALLBACK: bool> fmt::Debug for PathRouter<S, IS_FALLBACK> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PathRouter")
            .field("routes", &self.routes)
            .field("node", &self.node)
            .finish()
    }
}

impl<S, const IS_FALLBACK: bool> Clone for PathRouter<S, IS_FALLBACK> {
    fn clone(&self) -> Self {
        Self {
            routes: self.routes.clone(),
            node: self.node.clone(),
            prev_route_id: self.prev_route_id,
        }
    }
}

#[derive(Clone, Default)]
struct Node {
    inner: matchit::Router<RouteId>,
    route_id_to_path: HashMap<RouteId, Arc<str>>,
    path_to_route_id: HashMap<Arc<str>, RouteId>,
}

impl Node {
    fn insert(
        &mut self,
        path: impl Into<String>,
        val: RouteId,
    ) -> Result<(), matchit::InsertError> {
        let path = path.into();

        self.inner.insert(&path, val)?;

        let shared_path: Arc<str> = path.into();
        self.route_id_to_path.insert(val, shared_path.clone());
        self.path_to_route_id.insert(shared_path, val);

        Ok(())
    }

    fn at<'n, 'p>(
        &'n self,
        path: &'p str,
    ) -> Result<matchit::Match<'n, 'p, &'n RouteId>, MatchError> {
        self.inner.at(path)
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("paths", &self.route_id_to_path)
            .finish()
    }
}

#[track_caller]
fn validate_nest_path(path: &str) -> &str {
    if path.is_empty() {
        return "/";
    }

    if path.contains('*') {
        panic!("Invalid route: nested routes cannot contain wildcards (*)");
    }

    path
}

pub(crate) fn path_for_nested_route<'a>(prefix: &'a str, path: &'a str) -> Cow<'a, str> {
    debug_assert!(prefix.starts_with('/'));
    debug_assert!(path.starts_with('/'));

    if prefix.ends_with('/') {
        format!("{prefix}{}", path.trim_start_matches('/')).into()
    } else if path == "/" {
        prefix.into()
    } else {
        format!("{prefix}{path}").into()
    }
}