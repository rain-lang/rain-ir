/*!
`rain` value lifetimes
*/
use std::cmp::Ordering;

use crate::region::{Region, RegionBorrow};

/// A `rain` lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Lifetime(Region);

impl Lifetime {
    /// Borrow this lifetime
    #[inline]
    pub fn borrow_lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow(self.0.borrow_region())
    }
    /// Get the region of this lifetime
    #[inline]
    pub fn region(&self) -> RegionBorrow {
        self.0.borrow_region()
    }
    /// Check whether this lifetime is the static (null) lifetime
    #[inline]
    pub fn is_static(&self) -> bool {
        self.0.is_null()
    }
    /// Get the region-depth of this lifetime
    #[inline]
    pub fn depth(&self) -> usize {
        self.0.depth()
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
        Lifetime(region)
    }
}

impl PartialOrd for Lifetime {
    /**
    We define a lifetime to be a sublifetime of another lifetime if every value in one lifetime lies in the other,
    This naturally induces a partial ordering on the set of lifetimes.
    */
    fn partial_cmp(&self, other: &Lifetime) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

/// A borrow of a `rain` lifetime
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct LifetimeBorrow<'a>(RegionBorrow<'a>);

impl PartialOrd for LifetimeBorrow<'_> {
    /**
    We define a lifetime to be a sublifetime of another lifetime if every value in one lifetime lies in the other,
    This naturally induces a partial ordering on the set of lifetimes
    */
    fn partial_cmp(&self, other: &LifetimeBorrow<'_>) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<'a> LifetimeBorrow<'a> {
    /// Clone this lifetime
    #[inline]
    pub fn clone_lifetime(&self) -> Lifetime {
        Lifetime(self.0.clone_region())
    }
    /// Get the region of this lifetime
    #[inline]
    pub fn region(&self) -> RegionBorrow<'a> {
        self.0
    }
    /// Check whether this lifetime is the static (null) lifetime
    #[inline]
    pub fn is_static(&self) -> bool {
        self.0.is_null()
    }
    /// Get the region-depth of this lifetime
    #[inline]
    pub fn depth(&self) -> usize {
        self.0.depth()
    }
}

impl<'a> From<RegionBorrow<'a>> for LifetimeBorrow<'a> {
    #[inline]
    fn from(borrow: RegionBorrow) -> LifetimeBorrow {
        LifetimeBorrow(borrow)
    }
}

/// A trait implemented by values which have a lifetime
pub trait Live {
    /// Get the lifetime of this value
    fn lifetime(&self) -> LifetimeBorrow;
    /// Get the region of this value, or `None` if the value is global
    fn region(&self) -> RegionBorrow {
        self.lifetime().region()
    }
}
