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
    idempotent: true,
};

/// The data describing a `rain` lifetime
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LifetimeData {
    /// The affine members of this lifetime
    affine: Option<HashMap<Color, Affine>>,
    /// The region of this lifetime
    region: Region,
    /// Whether this lifetime is self-intersectable
    idempotent: bool,
}

impl LifetimeData {
    /// A helper function to take the separating conjunction of two affine lifetime sets
    #[inline]
    pub fn affine_star(
        left: HashMap<Color, Affine>,
        right: HashMap<Color, Affine>,
    ) -> Result<HashMap<Color, Affine>, Error> {
        let mut has_error: Option<Error> = None;
        let result =
            left.symmetric_difference_with(right, |left, right| match left.star(&right) {
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
    /// Check whether this lifetime is idempotent under separating conjunction, i.e.
    /// the separating conjunction of this lifetime with itself is itself
    #[inline]
    pub fn idempotent(&self) -> bool {
        self.idempotent
    }
    /// Get the separating conjunction of this lifetime with itself
    #[inline]
    pub fn star_self(&self) -> Result<(), Error> {
        if self.idempotent() {
            Ok(())
        } else {
            Err(Error::LifetimeError)
        }
    }
    /// Get the separating conjunction of this lifetime with another
    #[inline]
    pub fn star(&self, other: &LifetimeData) -> Result<LifetimeData, Error> {
        use Ordering::*;
        let region = match self.region.partial_cmp(&other.region) {
            None => return Err(Error::IncomparableRegions),
            Some(Less) | Some(Equal) => self.region.clone(),
            Some(Greater) => other.region.clone(),
        };
        let affine = match (self.affine.as_ref(), other.affine.as_ref()) {
            (l, None) => l.cloned(),
            (None, r) => r.cloned(),
            (Some(l), Some(r)) => {
                if HashMap::ptr_eq(l, r) {
                    if self.idempotent() {
                        Some(l.clone())
                    } else {
                        return Err(Error::LifetimeError);
                    }
                } else {
                    Some(Self::affine_star(l.clone(), r.clone())?)
                }
            }
        };
        let idempotent = self.idempotent & other.idempotent;
        Ok(LifetimeData {
            region,
            affine,
            idempotent,
        })
    }
}

/// The data describing an affine lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Affine {
    /// Owns an affine type completely
    Owned,
    /// Borrows an affine type from a source
    Borrowed(Borrowed),
    //TODO: own/borrow field set
}

impl Affine {
    /// Take the separating conjunction of this lifetime with another
    pub fn star(&self, other: &Affine) -> Result<Affine, Error> {
        use Affine::*;
        match (self, other) {
            (Borrowed(l), Borrowed(r)) => l.star(r).map(Borrowed),
            _ => Err(Error::LifetimeError),
        }
    }
    /// Whether this lifetime is idempotent under separating conjunction
    pub fn idempotent(&self) -> bool {
        use Affine::*;
        match self {
            Owned => false,
            Borrowed(_) => true,
        }
    }
    /// Take the separating conjunction of this affine lifetime with itself
    pub fn star_self(&self) -> Result<(), Error> {
        use Affine::*;
        match self {
            Owned => Err(Error::LifetimeError),
            Borrowed(_) => Ok(()),
        }
    }
}

/// An owned affine lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Owned {
    //TODO: optional source + field-set
}

/// A completely borrowed affine lifetime with a given source
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Borrowed(pub ValId);

impl Borrowed {
    /// Take the conjunction of this lifetime with itself
    #[inline]
    pub fn conj(&self, other: &Borrowed) -> Result<Borrowed, Error> {
        if self.0 == other.0 {
            Ok(self.clone())
        } else {
            Err(Error::LifetimeError)
        }
    }
    /// Take the separating conjunction of this lifetime with itself
    #[inline]
    pub fn star(&self, other: &Borrowed) -> Result<Borrowed, Error> {
        self.conj(other)
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
            idempotent: true,
        }
    }
}

impl Regional for LifetimeData {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.region.borrow_region()
    }
}
