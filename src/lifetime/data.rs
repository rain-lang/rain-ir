/*!
Lifetime data
*/
use super::*;
use crate::region::{Region, RegionBorrow, Regional};
use im::HashMap;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use crate::value::ValId;

/// The static `rain` lifetime, with a constant address
pub static STATIC_LIFETIME: LifetimeData = LifetimeData {
    affine: None,
    region: Region::NULL,
};

/// The data describing a `rain` lifetime
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LifetimeData {
    affine: Option<HashMap<Color, Affine>>,
    region: Region,
}

/// The data describing an affine lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Affine {
    /// Owns an affine type
    Owned(Owned),
    /// Borrows an affine type
    Borrowed(Borrowed)
}

/// An owned affine lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Owned {

}

/// A borrowed affine lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Borrowed {
    /// The source of the borrow
    pub source: ValId
}

impl Hash for LifetimeData {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.region.hash(hasher);
    }
}

impl PartialOrd for LifetimeData {
    fn partial_cmp(&self, other: &LifetimeData) -> Option<Ordering> {
        self.region.partial_cmp(&other.region)
    }
}

impl From<Region> for LifetimeData {
    #[inline]
    fn from(region: Region) -> LifetimeData {
        LifetimeData {
            affine: None,
            region,
        }
    }
}

impl Regional for LifetimeData {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.region.borrow_region()
    }
}
