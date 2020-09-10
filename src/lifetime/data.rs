/*!
Data describing a `rain` lifetime
*/
use super::*;
use crate::value::Error;

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
    /// The transient component of this lifetime, if any
    transient: Option<Group>,
    /// The lifetime parameters of this value, if any
    lt_params: LifetimeParams,
}

impl LifetimeData {
    /// Construct a new trivial lifetime from a region
    #[inline]
    pub fn from_region(region: Region) -> LifetimeData {
        LifetimeData {
            region,
            lender: None,
            transient: None,
            lt_params: LifetimeParams::default(),
        }
    }
    /// Construct a new transient lifetime from an optional group and a base region
    #[inline]
    pub fn new_transient(region: Region, transient: Option<Group>) -> Result<LifetimeData, Error> {
        unimplemented!("New transient construction")
    }
    /// Construct a new transient lifetime from an optional group
    #[inline]
    pub fn from_transient(transient: Option<Group>) -> Result<LifetimeData, Error> {
        Self::new_transient(Region::NULL, transient)
    }
    /// Check if lifetime data is trivial, i.e. consists only of region data
    #[inline]
    pub fn is_trivial(&self) -> bool {
        self.lender.is_none() && self.transient.is_none() && self.lt_params.is_empty()
    }
    /// Check if this lifetime is purely transient
    #[inline]
    pub fn is_transient(&self) -> bool {
        self.lender.is_none() && self.lt_params.is_empty()
    }
    /// Check if this lifetime is purely concrete
    #[inline]
    pub fn is_concrete(&self) -> bool {
        self.transient.is_none()
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
    /// Get the lender of this lifetime, if any
    #[inline]
    pub fn lender(&self) -> Option<&Group> {
        self.lender.as_ref()
    }
    /// Get the transient component of this lifetime, if any
    #[inline]
    pub fn transient(&self) -> Option<&Group> {
        self.transient.as_ref()
    }
    /// Get the non-transient component of this lifetime, if any
    #[inline]
    pub fn concrete(&self) -> LifetimeData {
        LifetimeData {
            region: self.region.clone(),
            lender: self.lender.clone(),
            transient: None,
            lt_params: self.lt_params.clone(),
        }
    }
    /// Get the lifetime parameters of this lifetime
    #[inline]
    pub fn params(&self) -> &LifetimeParams {
        &self.lt_params
    }
}

impl From<Region> for LifetimeData {
    #[inline]
    fn from(region: Region) -> LifetimeData {
        LifetimeData::from_region(region)
    }
}

impl Regional for LifetimeData {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.region.region()
    }
}
