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

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

pub(crate) type BoxError = Box<dyn std::error::Error>;

pub struct Env {
    pub polyfill_base: PathBuf,
    pub cache_dir: Option<PathBuf>,
    /// Set of cache keys that result in empty polyfill bundles (stored in memory for fast lookup)
    pub empty_cache_keys: Arc<RwLock<HashSet<String>>>,
    /** 不需要polyfill的UA */
    pub up_to_date_ua_metric: prometheus::IntCounter,
}
