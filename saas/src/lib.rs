#![warn(
    clippy::all,
    clippy::todo,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::mem_forget,
    clippy::unused_self,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::if_let_mutex,
    clippy::mismatched_target_os,
    clippy::await_holding_lock,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::exit,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::option_option,
    clippy::verbose_file_reads,
    clippy::unnested_or_patterns,
    clippy::str_to_string,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
    missing_debug_implementations,
    missing_docs
)]
#![deny(unreachable_pub, private_in_public)]
#![allow(elided_lifetimes_in_paths, clippy::type_complexity)]
// can't be `forbid` since we've vendored code from hyper-util that contains `unsafe`
// when hyper-util is on crates.io we can stop vendoring it and go back to `forbid`
#![deny(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![cfg_attr(test, allow(clippy::float_cmp))]
#![cfg_attr(not(test), warn(clippy::print_stdout, clippy::dbg_macro))]

#[macro_use]
pub(crate) mod macros;

mod boxed;
mod extension;

#[cfg(feature = "form")]
mod form;
#[cfg(feature = "tokio")]
mod hyper1_tokio_io;

#[cfg(feature = "json")]
mod json;

mod service_ext;
mod util;

pub mod body;
pub mod error_handling;
pub mod extract;
pub mod handler;
pub mod middleware;
pub mod response;
pub mod routing;
#[cfg(feature="tokio")]
pub mod serve;

#[cfg(test)]
mod test_helpers;

#[doc(no_inline)]
pub use async_trait::async_trait;

#[doc(no_inline)]
pub use http;
#[doc(inline)]
#[cfg(feature = "json")]
pub use crate::json::Json;
#[doc(inline)]
pub use self::routing::Router;

#[doc(inline)]
pub use crate::extension::Extension;

pub use saas_core::{BoxError, Error, RequestExt, RequestPartsExt};

#[cfg(feature = "tokio")]
#[doc(inline)]
pub use self::serve::serve;

pub use self::service_ext::ServiceExt;
