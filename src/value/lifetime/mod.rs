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
    #[inline] pub fn borrow_lifetime(&self) -> LifetimeBorrow { LifetimeBorrow(self.0.as_ref().map(Region::borrow_region)) }
}

impl Regional for Lifetime {
    fn region(&self) -> Option<RegionBorrow> { self.0.as_ref().map(Region::borrow_region) }
}

/// A borrow of a `rain` lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct LifetimeBorrow<'a>(Option<RegionBorrow<'a>>);

/// A trait implemented by values which have a lifetime
/// 
/// The region of the lifetime returned should be at most the size of the region returned by the 
/// `Regional` implementation, though the latter may be larger.
pub trait Live: Regional {
    /// Get the lifetime of this value
    fn lifetime(&self) -> Lifetime;
}