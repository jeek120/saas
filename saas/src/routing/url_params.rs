use std::sync::Arc;

use http::Extensions;
use matchit::Params;

use crate::util::PercentDecodedStr;

pub(crate) enum UrlParams {
    Params(Vec<(Arc<str>, PercentDecodedStr)>),
    InvalidUtf8InPathParams { key: Arc<str>},
}

pub(crate) fn insert_url_params(extensions: &mut Extensions, params: Params) {
    let current_params = extensions.get_mut();
    if let Some(UrlParams::InvalidUtf8InPathParams { .. }) = current_params {
        // 这里什么都不要做，之前就有错误
        return;
    }

    let params = params
        .iter()
        .filter(|(key, _)| !key.starts_with(super::NEST_TAIL_PARAM))
        .filter(|(key, _)| !key.starts_with(super::FALLBACK_PARAM))
        .map(|(k,v)| {
            if let Some(decode) = PercentDecodedStr::new(v) {
                Ok((Arc::from(k), decode))
            }else{
                Err(Arc::from(k))
            }
        })
        .collect::<Result<Vec<_>, _>>();

    match (current_params, params) {
        (Some(UrlParams::InvalidUtf8InPathParams { .. }), _) => {
            unreachable!("we check for this state earlier in this method")
        }
        (_, Err(invalid_key)) => {
            extensions.insert(UrlParams::InvalidUtf8InPathParams { key: invalid_key });
        }
        (Some(UrlParams::Params(current)), Ok(params)) => {
            current.extend(params);
        }
        (None,Ok(params)) => {
            extensions.insert(params);
        }
    }
}