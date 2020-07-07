/*!
Lifetime data
*/
use super::*;
use crate::region::{Region, RegionBorrow, Regional};
use crate::value::{Error, ValId};
use im::HashMap;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

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
    Borrowed(Borrowed),
}

impl Affine {
    /// Intersect this affine lifetime with another
    pub fn intersect(&self, other: &Affine) -> Result<Affine, Error> {
        use Affine::*;
        match (self, other) {
            (Borrowed(l), Borrowed(r)) => l.intersect(r).map(Borrowed),
            _ => Err(Error::LifetimeError),
        }
    }
}

/// An owned affine lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Owned {
    //TODO: optional source + field-set
}

/// A borrowed affine lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Borrowed {
    /// The source of the borrow
    pub source: ValId, //TODO: optional field-set
}

impl Borrowed {
    /// Intersect this borrowed lifetime with another
    pub fn intersect(&self, other: &Borrowed) -> Result<Borrowed, Error> {
        if self.source == other.source {
            Ok(self.clone())
        } else {
            Err(Error::LifetimeError)
        }
    }
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
