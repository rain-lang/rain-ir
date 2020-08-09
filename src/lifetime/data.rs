/*!
Lifetime data
*/
use super::*;
use crate::region::{Region, RegionBorrow, Regional};
use crate::typing::Type;
use crate::value::Error;
use lazy_static::lazy_static;
use std::cmp::Ordering;
use std::hash::Hash;

lazy_static! {
    /// The static `rain` lifetime, with a constant address
    pub static ref STATIC_LIFETIME_DATA: LifetimeData = LifetimeData::default();
}

/// Get a reference to the static affine lifetime
pub fn static_affine_lifetime() -> &'static AffineData {
    &STATIC_LIFETIME_DATA.affine
}

/// Get a reference to the static relevant lifetime
pub fn static_relevant_lifetime() -> &'static RelevantData {
    &STATIC_LIFETIME_DATA.relevant
}

/// The data describing a `rain` lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct LifetimeData {
    /// The affine component of this lifetime
    affine: AffineData,
    /// The relevant component of this lifetime
    relevant: RelevantData,
    /// The region of this lifetime
    pub(super) region: Option<Region>,
}

impl LifetimeData {
    /// Try to create a purely affine lifetime
    ///
    /// Fails if the region is inconsistent
    #[inline]
    pub fn try_from_affine(affine: AffineData) -> Result<LifetimeData, Error> {
        let region = affine.region()?.cloned_region();
        Ok(LifetimeData {
            affine,
            region,
            relevant: RelevantData::default(),
        })
    }
    /// Create a lifetime which only owns a particular color
    #[inline]
    pub fn owns(color: Color) -> LifetimeData {
        let affine = AffineData::owns(color);
        Self::try_from_affine(affine).expect("Single color lifetimes always have valid regions")
    }
    /// Gets the lifetime for the nth parameter of a `Region`.
    ///
    /// Returns an error on index out of bounds
    #[inline]
    pub fn param(region: Region, ix: usize) -> Result<LifetimeData, Error> {
        let ty = region.param_tys().get(ix).ok_or(Error::InvalidParam)?;
        let mut relevant = RelevantData::default();
        let affine = if ty.is_affine() {
            AffineData::owns(Color::param_unchecked(region.clone(), ix))
        } else {
            AffineData::default()
        };
        if ty.is_relevant() {
            relevant
                .data
                .insert(Color::param_unchecked(region.clone(), ix), Relevant::Used);
        }
        Ok(LifetimeData {
            affine,
            relevant,
            region: Some(region),
        })
    }
    /// Whether this lifetime is static
    #[inline]
    pub fn is_static(&self) -> bool {
        self.region.is_none() && self.affine.is_static() && self.relevant.is_static()
    }
    /// Whether this lifetime is affine
    #[inline]
    pub fn is_affine(&self) -> bool {
        self.affine.is_affine()
    }
    /// Whether this lifetime is relevant
    #[inline]
    pub fn is_relevant(&self) -> bool {
        self.relevant.is_relevant()
    }
    /// Whether this lifetime is linear
    #[inline]
    pub fn is_linear(&self) -> bool {
        self.is_affine() && self.is_relevant()
    }
    /// Whether this lifetime is substructural
    #[inline]
    pub fn is_substruct(&self) -> bool {
        self.is_affine() || self.is_relevant()
    }
    /// Get the separating conjunction of two lifetimes
    #[inline]
    pub fn sep_conj(&self, other: &LifetimeData) -> Result<LifetimeData, Error> {
        let region = self.gcr(other)?.cloned_region();
        let affine = (&self.affine * &other.affine)?;
        let relevant = &self.relevant * &other.relevant;
        Ok(LifetimeData {
            affine,
            relevant,
            region,
        })
    }
    /// Get the disjunction of two lifetimes
    #[inline]
    pub fn disj(&self, other: &LifetimeData) -> Result<LifetimeData, Error> {
        let region = self.gcr(other)?.cloned_region();
        let affine = (&self.affine + &other.affine)?;
        let relevant = &self.relevant + &other.relevant;
        Ok(LifetimeData {
            affine,
            relevant,
            region,
        })
    }
    /// Get a reference to the affine component of this lifetime
    #[inline]
    pub fn affine(&self) -> &AffineData {
        &self.affine
    }
    /// Get a reference to the relevant component of this lifetime
    #[inline]
    pub fn relevant(&self) -> &RelevantData {
        &self.relevant
    }
    /// Get the affine component of this lifetime
    #[inline]
    pub fn affine_component(&self) -> LifetimeData {
        LifetimeData {
            affine: self.affine.clone(),
            relevant: RelevantData::default(),
            region: self.region.clone(),
        }
    }
    /// Get the relevant component of this lifetime
    #[inline]
    pub fn relevant_component(&self) -> LifetimeData {
        LifetimeData {
            affine: AffineData::default(),
            relevant: self.relevant.clone(),
            region: self.region.clone(),
        }
    }
    /// Get this lifetime data, but within a given region
    #[inline]
    pub fn in_region(&self, region: Option<Region>) -> Result<LifetimeData, Error> {
        if self.region <= region {
            Ok(LifetimeData {
                affine: self.affine.clone(),
                relevant: self.relevant.clone(),
                region,
            })
        } else {
            Err(Error::IncomparableRegions)
        }
    }
    /// Attempt to color map a lifetime while truncating it's region to a given level
    ///
    /// Leaves the lifetime in an undetermined but valid state on failure
    #[inline]
    pub fn color_map<'a, F, P>(
        &mut self,
        mut color_map: F,
        mut parametric_map: P,
        depth: usize,
    ) -> Result<(), Error>
    where
        F: FnMut(&Color) -> Option<&'a Lifetime>,
        P: FnMut(&ValId) -> Result<ValId, Error>,
    {
        self.region = self
            .region
            .ancestor(depth.saturating_sub(1))
            .cloned_region();
        self.affine
            .color_map(&mut color_map, &mut parametric_map, depth)?;
        self.relevant.color_map(&mut color_map, depth)?;
        Ok(())
    }
}

impl PartialOrd for LifetimeData {
    fn partial_cmp(&self, other: &LifetimeData) -> Option<Ordering> {
        unimplemented!("Lifetime data ordering: {:#?}, {:#?}", self, other)
    }
}

impl From<Option<Region>> for LifetimeData {
    #[inline]
    fn from(region: Option<Region>) -> LifetimeData {
        LifetimeData {
            affine: AffineData::default(),
            relevant: RelevantData::default(),
            region,
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
