/*!
Lifetime data
*/
use super::*;
use crate::region::{Region, RegionBorrow, Regional};
use crate::value::{Error, ValId};
use im::{hashmap, HashMap};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// The static `rain` lifetime, with a constant address
pub static STATIC_LIFETIME: LifetimeData = LifetimeData {
    affine: None,
    region: Region::NULL,
    idempotent: true,
};

/// The data describing a `rain` lifetime
#[derive(Debug, Clone, Eq)]
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
        let result = left.symmetric_difference_with(right, |left, right| match left.star(&right) {
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
    /// A helper function to take the conjunction of two affine lifetime sets
    #[inline]
    pub fn affine_conj(
        left: HashMap<Color, Affine>,
        right: HashMap<Color, Affine>,
    ) -> Result<HashMap<Color, Affine>, Error> {
        let mut has_error: Option<Error> = None;
        let result = left.symmetric_difference_with(right, |left, right| match left.conj(&right) {
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
    ///
    /// Note that *every* lifetime is idempotent under *conjunction*, so there is no need to
    /// add a helper to check this.
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
    /// Get the conjunction of this lifetime with itself
    #[inline]
    pub fn conj(&self, other: &LifetimeData) -> Result<LifetimeData, Error> {
        let region = self.region.conj(&other.region)?;
        let affine = match (self.affine.as_ref(), other.affine.as_ref()) {
            (l, None) => l.cloned(),
            (None, r) => r.cloned(),
            (Some(l), Some(r)) => {
                if HashMap::ptr_eq(l, r) {
                    Some(l.clone())
                } else {
                    Some(Self::affine_conj(l.clone(), r.clone())?)
                }
            }
        };
        let idempotent = self.idempotent & other.idempotent;
        Ok(LifetimeData {
            region: region.clone(),
            affine,
            idempotent,
        })
    }
    /// Apply the borrow transformation to this lifetime at a given `ValId`
    #[inline]
    pub fn borrowed(self, source: ValId) -> LifetimeData {
        if self.idempotent {
            return self;
        }
        let mut affine = if let Some(affine) = self.affine {
            affine
        } else {
            // Should be idempotent in this case... consider panicking...
            return self;
        };
        //TODO: optimize memory usage?
        for (_key, value) in affine.iter_mut() {
            *value = value.borrowed(source.clone());
        }
        LifetimeData {
            affine: Some(affine),
            region: self.region,
            idempotent: true,
        }
    }
    /// Get the separating conjunction of this lifetime with another
    #[inline]
    pub fn star(&self, other: &LifetimeData) -> Result<LifetimeData, Error> {
        let region = self.region.conj(&other.region)?;
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
            region: region.clone(),
            affine,
            idempotent,
        })
    }
    /// Gets a lifetime which only owns a given color
    #[inline]
    pub fn owns(color: Color) -> LifetimeData {
        let region = color.region().clone_region();
        // Not idempotent since owned
        LifetimeData {
            affine: Some(hashmap! { color => Affine::Owned }),
            region,
            idempotent: false,
        }
    }
    /// Gets the lifetime for the nth parameter of a `Region`. Returns a blank lifetime LifetimeData on OOB
    #[inline]
    pub fn param(region: Region, ix: usize) -> LifetimeData {
        if let Some(color) = Color::param(region.clone(), ix) {
            LifetimeData::owns(color)
        } else {
            LifetimeData::from(region)
        }
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
    /// Borrow this lifetime at a given source point
    pub fn borrowed(&self, source: ValId) -> Affine {
        match self {
            Affine::Owned => Affine::Borrowed(Borrowed(source)),
            b => b.clone(),
        }
    }
    /// Take the separating conjunction of this lifetime with another
    pub fn star(&self, other: &Affine) -> Result<Affine, Error> {
        use Affine::*;
        match (self, other) {
            (Borrowed(l), Borrowed(r)) => l.star(r).map(Borrowed),
            _ => Err(Error::LifetimeError),
        }
    }
    /// Take the conjunction of this lifetime with another
    pub fn conj(&self, other: &Affine) -> Result<Affine, Error> {
        use Affine::*;
        match (self, other) {
            (Owned, _) => Ok(Owned),
            (_, Owned) => Ok(Owned),
            (Borrowed(l), Borrowed(r)) => l.conj(r).map(Borrowed),
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
        self.affine.hash(hasher);
    }
}

impl PartialEq for LifetimeData {
    fn eq(&self, other: &LifetimeData) -> bool {
        self.region == other.region && self.affine == other.affine
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_affine_lifetime_operations_work() {
        let red = Color::new();
        let yellow = Color::new();
        let alpha = LifetimeData::owns(red);
        let beta = LifetimeData::owns(yellow);
        assert!(!alpha.idempotent());
        assert!(!beta.idempotent());
        assert_eq!(alpha, alpha);
        assert_eq!(beta, beta);
        assert_ne!(alpha, beta);
        let gamma = alpha.star(&beta).expect("Valid lifetime");
        assert!(!gamma.idempotent());
        let gamma_ = alpha.conj(&beta).expect("Valid lifetime");
        assert_eq!(gamma, gamma_);
        assert_ne!(gamma, alpha);
        assert_ne!(gamma, beta);
        gamma.star(&alpha).expect_err("Gamma owns alpha");
        gamma.star(&beta).expect_err("Gamma owns beta");
        assert_eq!(
            gamma
                .conj(&alpha)
                .expect("Gamma owns alpha, but a branch is OK"),
            gamma
        );
        let a = alpha.clone().borrowed(().into());
        assert!(a.idempotent());
        assert_eq!(a, a);
        assert_ne!(alpha, a);
        assert_eq!(a.star(&a).expect("Borrows are idempotent"), a);
        a.star(&alpha).expect_err("Alpha owns what a borrows");
        assert_eq!(a.conj(&alpha).expect("Borrow | Affine = Affine"), alpha);
        assert_eq!(alpha.conj(&a).expect("Affine | Borrow = Affine"), alpha);
    }
}
