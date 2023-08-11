use std::{sync::Arc, collections::HashMap};

use async_trait::async_trait;
use http::request::Parts;
use saas_core::extract::{FromRequestParts};

use crate::routing::{RouteId, NEST_TAIL_PARAM_CAPTURE};

use super::rejection::{MatchedPathRejection, MatchedPathMissing};



#[cfg_attr(docsrs, doc(cfg(feature = "matched-path")))]
#[derive(Clone, Debug)]
pub struct MatchedPath(pub(crate) Arc<str>);

impl MatchedPath {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for MatchedPath
where
    S: Send + Sync,
{
    type Rejection = MatchedPathRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let matched_path = parts
            .extensions
            .get::<Self>()
            .ok_or(MatchedPathRejection::MatchedPathMissing(MatchedPathMissing))?
            .clone();
    
        Ok(matched_path)
    }
}

#[derive(Clone, Debug)]
struct MatchedNestedPath(Arc<str>);

pub(crate) fn set_matched_path_for_request(
    id: RouteId,
    route_id_to_path: &HashMap<RouteId, Arc<str>>,
    extensions: &mut http::Extensions,
) {
    let matched_path = if let Some(matched_path) = route_id_to_path.get(&id) {
        matched_path
    } else {
        #[cfg(debug_assertions)]
        panic!("should always hava a matched path for a route id");
        #[cfg(not(debug_assertions))]
        return;
    };

    let matched_path = append_nested_matched_path(matched_path, extensions);

    if matched_path.ends_with(NEST_TAIL_PARAM_CAPTURE) {
        extensions.insert(MatchedNestedPath(matched_path));
        debug_assert!(extensions.remove::<MatchedPath>().is_none());
    } else {
        extensions.insert(MatchedPath(matched_path));
        extensions.remove::<MatchedNestedPath>();
    }
}

fn append_nested_matched_path(matched_path: &Arc<str>, extensions: &http::Extensions) -> Arc<str> {
    if let Some(previous) = extensions
        .get::<MatchedPath>()
        .map(|matched_path| matched_path.as_str())
        .or_else(|| Some(&extensions.get::<MatchedNestedPath>()?.0))
    {
        let previous = previous
            .strip_suffix(NEST_TAIL_PARAM_CAPTURE)
            .unwrap_or(previous);

        let matched_path = format!("{previous}{matched_path}");
        matched_path.into()
    } else {
        Arc::clone(matched_path)
    }
}

