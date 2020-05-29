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
}

/// A borrow of a `rain` lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct LifetimeBorrow<'a>(Option<RegionBorrow<'a>>);

impl<'a> LifetimeBorrow<'a> {
    /// Get the region of this lifetime
    #[inline]
    pub fn region(&self) -> Option<RegionBorrow<'a>> {
        self.0
    }
}

/// A trait implemented by values which have a lifetime
pub trait Live {
    /// Get the lifetime of this value
    fn lifetime(&self) -> Lifetime;
}
