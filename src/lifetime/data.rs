/*!
Lifetime data
*/
use super::*;
use crate::region::{lcr, Region, RegionBorrow, Regional};
use crate::typing::Type;
use crate::value::Error;
use lazy_static;
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
    /// Gets the lifetime for the nth parameter of a `Region`. Returns a regular lifetime `Region` on OOB
    #[inline]
    pub fn param(region: Region, ix: usize) -> Result<LifetimeData, Error> {
        let ty = region.param_tys().get(ix).ok_or(Error::InvalidParam)?;
        let mut affine = AffineData::default();
        let mut relevant = RelevantData::default();
        if ty.is_affine() {
            affine.affine = true;
            affine
                .data
                .insert(Color::param_unchecked(region.clone(), ix), Affine::Owned);
        }
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
    /// Get the separating conjunction of two lifetimes
    #[inline]
    pub fn sep_conj(&self, other: &LifetimeData) -> Result<LifetimeData, Error> {
        let region = self.lcr(other)?.cloned_region();
        //TODO: size optimizations?
        let mut affine = self.affine.clone();
        affine.sep_conj(&other.affine)?;
        let relevant = &self.relevant * &other.relevant;
        Ok(LifetimeData {
            affine,
            relevant,
            region,
        })
    }
    /*
    /// Get the disjunction of two lifetimes
    #[inline]
    pub fn disj(&self, other: &LifetimeData) -> Result<LifetimeData, Error> {

    }
    /// Get the affine component of this lifetime
    #[inline]
    pub fn affine_component(&self) -> LifetimeData {

    }
    /// Get the relevant component of this lifetime
    #[inline]
    pub fn relevant_component(&self) -> LifetimeData {

    }
    */
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
}

impl PartialOrd for LifetimeData {
    fn partial_cmp(&self, other: &LifetimeData) -> Option<Ordering> {
        unimplemented!("Lifetime data ordering")
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
