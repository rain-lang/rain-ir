/*!
`rain` value lifetimes
*/

mod region;
pub use region::*;

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
    /// Find the intersection of a set of lifetimes and this lifetime. Return an error if the lifetimes are incompatible.
    #[inline]
    pub fn intersect<'a>(
        &'a self,
        lifetimes: &'a [LifetimeBorrow<'a>],
    ) -> Result<LifetimeBorrow<'a>, ()> {
        let mut base = self.borrow_lifetime();
        for lifetime in lifetimes.iter().copied() {
            if base.is_static() {
                base = lifetime
            }
            if !lifetime.is_static() {
                unimplemented!()
            }
        }
        Ok(base)
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

/// A borrow of a `rain` lifetime
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct LifetimeBorrow<'a>(Option<RegionBorrow<'a>>);

impl<'a> LifetimeBorrow<'a> {
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
}
