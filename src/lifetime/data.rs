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

impl LifetimeData {
    /// A helper function to intersect two affine lifetime sets
    #[inline]
    pub fn affine_intersect(
        left: HashMap<Color, Affine>,
        right: HashMap<Color, Affine>,
    ) -> Result<HashMap<Color, Affine>, Error> {
        let mut has_error: Option<Error> = None;
        let result =
            left.symmetric_difference_with(right, |left, right| match left.intersect(&right) {
                Ok(int) => Some(int),
                Err(err) => {
                    has_error = Some(err);
                    None
                }
            });
        if let Some(err) = has_error {
            return Err(err);
        }
        Ok(result)
    }
    /// A helper function to intersect an affine lifetime set with itself
    #[inline]
    pub fn affine_intersect_self(affine: &HashMap<Color, Affine>) -> Result<(), Error> {
        for (_, affine) in affine.iter() {
            affine.intersect_self()?
        }
        Ok(())
    }
    /// Intersect this lifetime data with itself
    #[inline]
    pub fn intersect_self(&self) -> Result<(), Error> {
        if let Some(affine) = &self.affine {
            Self::affine_intersect_self(affine)
        } else {
            Ok(())
        }
    }
    /// Intersect this lifetime data with another
    #[inline]
    pub fn intersect(&self, other: &LifetimeData) -> Result<LifetimeData, Error> {
        use Ordering::*;
        let region = match self.region.partial_cmp(&other.region) {
            None => return Err(Error::IncomparableRegions),
            Some(Less) | Some(Equal) => self.region.clone(),
            Some(Greater) => other.region.clone(),
        };
        let affine = match (self.affine.as_ref(), other.affine.as_ref()) {
            (l, None) => l.cloned(),
            (None, r) => r.cloned(),
            _ => unimplemented!("Double-affine intersection"),
        };
        Ok(LifetimeData { region, affine })
    }
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
    /// Intersect this affine lifetime with itself
    pub fn intersect_self(&self) -> Result<(), Error> {
        use Affine::*;
        match self {
            Owned(_) => Err(Error::LifetimeError),
            Borrowed(_) => Ok(()),
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
