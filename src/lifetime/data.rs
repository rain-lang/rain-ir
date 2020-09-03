/*!
Data describing a `rain` lifetime
*/
use super::*;

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

impl LifetimeData {
    /// Construct a new trivial lifetime from a region
    #[inline]
    pub fn trivial(region: Region) -> LifetimeData {
        LifetimeData {
            region,
            lender: None,
            lt_params: LifetimeParams::default(),
        }
    }
    /// Check if lifetime data is trivial, i.e. consists only of region data
    #[inline]
    pub fn is_trivial(&self) -> bool {
        self.lender.is_none() && self.lt_params.is_empty()
    }
    /// Try to cast this lifetime into a nontrivial lifetime. On failure, return it's region
    #[inline]
    pub fn into_nontrivial(self) -> Result<LifetimeData, Region> {
        if self.is_trivial() {
            Err(self.region)
        } else {
            Ok(self)
        }
    }
}

impl From<Region> for LifetimeData {
    #[inline]
    fn from(region: Region) -> LifetimeData {
        LifetimeData::trivial(region)
    }
}

impl Regional for LifetimeData {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.region.region()
    }
}
