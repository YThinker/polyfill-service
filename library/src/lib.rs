#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
pub mod buffer;
pub mod features_from_query_parameter;
pub mod get_polyfill_string;
pub mod meta;
pub mod old_ua;
pub mod parse;
pub mod polyfill_parameters;
pub mod toposort;
pub mod ua;
pub mod useragent;

use std::path::PathBuf;

pub(crate) type BoxError = Box<dyn std::error::Error>;

pub struct Env {
    pub polyfill_base: PathBuf,
    pub d1_query_metric: prometheus::IntCounterVec,
    pub up_to_date_ua_metric: prometheus::IntCounter,
    pub injected_polyfill_metric: prometheus::IntCounter,
    pub bytes_out_metric: prometheus::IntCounter,
}
