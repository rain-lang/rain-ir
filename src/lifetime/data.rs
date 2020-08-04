/*!
Lifetime data
*/
use super::*;
use crate::region::{Region, RegionBorrow, Regional};
use crate::value::{Error, ValId};
use im::{hashmap, hashmap::Entry, HashMap};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// The static `rain` lifetime, with a constant address
pub static STATIC_LIFETIME: LifetimeData = LifetimeData {
    affine: None,
    region: None,
    idempotent: true,
};

/// The data describing a `rain` lifetime
#[derive(Debug, Clone, Eq)]
pub struct LifetimeData {
    /// The affine members of this lifetime
    affine: Option<HashMap<Color, Affine>>,
    /// The region of this lifetime
    pub(super) region: Option<Region>,
    /// Whether this lifetime is self-intersectable
    idempotent: bool,
}

impl LifetimeData {
    /// Get this lifetime data, but within a given region
    #[inline]
    pub fn in_region(&self, region: Option<Region>) -> Result<LifetimeData, Error> {
        if self.region <= region {
            Ok(LifetimeData {
                affine: self.affine.clone(),
                region,
                idempotent: self.idempotent,
            })
        } else {
            Err(Error::IncomparableRegions)
        }
    }
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
        let region = Region::conj(&self.region, &other.region)?;
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
    pub fn borrow_from(self, source: ValId) -> Result<LifetimeData, Error> {
        let mut affine = if let Some(affine) = self.affine {
            affine
        } else {
            // Should be idempotent in this case... consider panicking...
            return Ok(self);
        };
        //TODO: optimize memory usage?
        for (_key, value) in affine.iter_mut() {
            *value = value.borrow_from(source.clone())?;
        }
        Ok(LifetimeData {
            affine: Some(affine),
            region: self.region,
            idempotent: true,
        })
    }
    /// Get the separating conjunction of this lifetime with another
    #[inline]
    pub fn star(&self, other: &LifetimeData) -> Result<LifetimeData, Error> {
        let region = Region::conj(&self.region, &other.region)?;
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
        let region = color.cloned_region();
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
    /// Attempt to apply a color mapping to this lifetime data
    #[inline]
    pub fn color_map<'a, F>(&self, color_map: F, depth: usize) -> Result<LifetimeData, Error>
    where
        F: Fn(&Color) -> &'a [Color],
    {
        // Ignore shallow lifetimes
        if self.depth() < depth {
            return Ok(self.clone());
        }
        let region = self
            .region
            .ancestor(depth.saturating_sub(1))
            .cloned_region();
        let affine = if let Some(affine) = &self.affine {
            affine
        } else {
            return Ok(LifetimeData {
                affine: None,
                region,
                idempotent: true,
            });
        };
        let mut new_affine = affine.clone();
        // Set to false if self is idempotent to avoid checks
        let mut idempotent = !self.idempotent;
        for (color, affinity) in affine.iter() {
            // Filter out shallow colors, but check for non-idempotence
            if color.depth() < depth {
                if idempotent && !affinity.idempotent() {
                    idempotent = false
                }
                continue;
            }
            // Remove the given color's mapping
            new_affine.remove(color);
            // Insert target color mappings
            for target_color in color_map(color) {
                if idempotent && !affinity.idempotent() {
                    idempotent = false
                }
                match new_affine.entry(target_color.clone()) {
                    Entry::Occupied(mut o) => {
                        let new_affinity = o.get().star(affinity)?;
                        // Avoid unnecessary cloning in the immutable map!
                        if new_affinity != *o.get() {
                            *o.get_mut() = new_affinity
                        }
                    }
                    Entry::Vacant(v) => {
                        v.insert(affinity.clone());
                    }
                }
            }
        }
        // Now, set to true if self is idempotent as idempotent is preserved under color mappings
        idempotent |= self.idempotent;
        Ok(LifetimeData {
            affine: Some(new_affine),
            region,
            idempotent,
        })
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

impl From<Option<Region>> for LifetimeData {
    #[inline]
    fn from(region: Option<Region>) -> LifetimeData {
        LifetimeData {
            affine: None,
            region,
            idempotent: true,
        }
    }
}

impl From<Region> for LifetimeData {
    #[inline]
    fn from(region: Region) -> LifetimeData {
        LifetimeData::from(Some(region))
    }
}

impl Regional for LifetimeData {
    #[inline]
    fn region(&self) -> Option<RegionBorrow> {
        self.region.as_ref().map(|region| region.borrow_region())
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
        let a = alpha.clone().borrow_from(().into()).unwrap();
        assert!(a.idempotent());
        assert_eq!(a, a);
        assert_ne!(alpha, a);
        assert_eq!(a.star(&a).expect("Borrows are idempotent"), a);
        a.star(&alpha).expect_err("Alpha owns what a borrows");
        assert_eq!(a.conj(&alpha).expect("Borrow | Affine = Affine"), alpha);
        assert_eq!(alpha.conj(&a).expect("Affine | Borrow = Affine"), alpha);
    }
}
