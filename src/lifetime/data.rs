/*!
Data describing a `rain` lifetime
*/
use super::*;
use dashcache::DashCache;
use lazy_static::lazy_static;

lazy_static! {
    /// The global cache of constructed nontrivial lifetimes
    pub static ref LIFETIME_CACHE: DashCache<Arc<LifetimeData>> = DashCache::new();
}

/// Data describing a nontrivial `rain` lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LifetimeData {
    /// The base region of this lifetime
    region: Region,
    /// The lender of this value, if any
    lender: Option<Group>,
    /// The lifetime parameters of this value, if any
    lt_params: LifetimeParams,
}
