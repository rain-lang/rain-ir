/*!
`rain` value lifetimes
*/
use std::cmp::Ordering;

use crate::region::{Region, RegionBorrow};

/// A `rain` lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Lifetime(Option<Region>);

impl Lifetime {
    /// Borrow this lifetime
    #[inline]
    pub fn borrow_lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow(self.0.as_ref().map(Region::borrow_region))
    }
    /// Get the region of this lifetime
    #[inline]
    pub fn region(&self) -> Option<RegionBorrow> {
        self.borrow_lifetime().region()
    }
    /// Check whether this lifetime is the static (null) lifetime
    #[inline]
    pub fn is_static(&self) -> bool {
        self.0.is_none()
    }
    /// Get the region-depth of this lifetime
    #[inline]
    pub fn depth(&self) -> usize {
        self.0.as_ref().map(|region| region.depth()).unwrap_or(0)
    }
    /// Find the intersection of a set of lifetimes and this lifetime. Return an error if the lifetimes are incompatible.
    #[inline]
    pub fn intersect<'a, I>(&'a self, lifetimes: I) -> Result<Lifetime, ()>
    where
        I: Iterator<Item = LifetimeBorrow<'a>>,
    {
        let mut base = self.borrow_lifetime();
        for lifetime in lifetimes {
            if let Some(ord) = base.partial_cmp(&lifetime) {
                if ord == Ordering::Less {
                    base = lifetime
                }
            } else {
                //TODO: lifetime intersections where possible...
                return Err(()); // Incompatible regions!
            }
        }
        Ok(base.clone_lifetime())
    }
}

impl From<Region> for Lifetime {
    #[inline]
    fn from(region: Region) -> Lifetime {
        Lifetime(Some(region))
    }
}

impl From<Option<Region>> for Lifetime {
    #[inline]
    fn from(region: Option<Region>) -> Lifetime {
        Lifetime(region)
    }
}

impl PartialOrd for Lifetime {
    /**
    We define a lifetime to be a sublifetime of another lifetime if every value in one lifetime lies in the other,
    This naturally induces a partial ordering on the set of regions.
    */
    fn partial_cmp(&self, other: &Lifetime) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

/// A borrow of a `rain` lifetime
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct LifetimeBorrow<'a>(Option<RegionBorrow<'a>>);

impl PartialOrd for LifetimeBorrow<'_> {
    /**
    We define a lifetime to be a sublifetime of another lifetime if every value in one lifetime lies in the other,
    This naturally induces a partial ordering on the set of regions.
    */
    fn partial_cmp(&self, other: &LifetimeBorrow<'_>) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<'a> LifetimeBorrow<'a> {
    /// Clone this lifetime
    #[inline]
    pub fn clone_lifetime(&self) -> Lifetime {
        Lifetime(self.0.map(|r| r.clone_region()))
    }
    /// Get the region of this lifetime
    #[inline]
    pub fn region(&self) -> Option<RegionBorrow<'a>> {
        self.0
    }
    /// Check whether this lifetime is the static (null) lifetime
    #[inline]
    pub fn is_static(&self) -> bool {
        self.0.is_none()
    }
    /// Get the region-depth of this lifetime
    #[inline]
    pub fn depth(&self) -> usize {
        self.0.map(|region| region.depth()).unwrap_or(0)
    }
}

impl<'a> From<RegionBorrow<'a>> for LifetimeBorrow<'a> {
    #[inline]
    fn from(borrow: RegionBorrow) -> LifetimeBorrow {
        LifetimeBorrow(Some(borrow))
    }
}

impl<'a> From<Option<RegionBorrow<'a>>> for LifetimeBorrow<'a> {
    #[inline]
    fn from(borrow: Option<RegionBorrow>) -> LifetimeBorrow {
        LifetimeBorrow(borrow)
    }
}

/// A trait implemented by values which have a lifetime
pub trait Live {
    /// Get the lifetime of this value
    fn lifetime(&self) -> LifetimeBorrow;
    /// Get the region of this value, or `None` if the value is global
    fn region(&self) -> Option<RegionBorrow> {
        self.lifetime().region()
    }
}
