/*!
`rain` value regions
*/
use crate::value::{arr::TyArr, Error, TypeId};
use dashcache::{DashCache, GlobalCache};
use elysees::{Arc, ArcBorrow};
use lazy_static::lazy_static;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, Index};

pub mod data;

mod parametrized;
pub use parametrized::*;
mod parameter;
use data::*;
pub use parameter::*;
mod region_impl;

/// A `rain` region
#[derive(Debug, Clone, Eq)]
pub struct Region(Arc<RegionData>);

/// A borrow of a `rain` region
#[derive(Debug, Copy, Clone, Eq)]
pub struct RegionBorrow<'a>(ArcBorrow<'a, RegionData>);

/// A trait for objects which have a region
pub trait Regional {
    /// Get the region of this object
    #[inline]
    fn region(&self) -> Option<RegionBorrow> {
        None
    }
    /// Get the region of this object, cloned
    #[inline]
    fn cloned_region(&self) -> Option<Region> {
        self.region().map(|region| region.clone_region())
    }
    /// Get the depth of this object's region
    #[inline]
    fn depth(&self) -> usize {
        self.region().depth()
    }
    /// Get the ancestor of this region up to a given depth, or this value's region if `depth >= self.depth()`
    #[inline]
    fn ancestor(&self, depth: usize) -> Option<RegionBorrow> {
        if let Some(region) = self.region() {
            Some(
                region
                    .data()
                    .parents
                    .get(depth)
                    .map(|region| region.borrow_region())
                    .unwrap_or(region),
            )
        } else {
            None
        }
    }
}

lazy_static! {
    /// The global cache of constructed regions.
    ///
    /// Note: region caching is not actually necessary for correctness, so consider exponsing a constructor
    /// for `Region`/`RegionBorrow` from `Arc<RegionData>` and `Arc<Region>`...
    pub static ref REGION_CACHE: DashCache<Arc<RegionData>> = DashCache::new();
}

impl Region {
    /// Create a new reference from a given `RegionData`, caching if possible
    #[inline]
    pub fn new(data: RegionData) -> Region {
        Region(REGION_CACHE.cache(data))
    }
    /// Create a new region with a given parameter type vector and a parent region
    #[inline]
    pub fn with(param_tys: TyArr, parent: Option<Region>) -> Region {
        Region::new(RegionData::with(param_tys, parent))
    }
    /// Create a new region with a given parent region and no parameters
    #[inline]
    pub fn with_parent(parent: Option<Region>) -> Region {
        Region::new(RegionData::with_parent(parent))
    }
    /// Get a reference to a borrow of this region. More efficient than taking an `&Region`.
    #[inline]
    pub fn borrow_region(&self) -> RegionBorrow {
        RegionBorrow(self.0.borrow_arc())
    }
    /// Get the underlying `Arc` of this `Region`, if any
    #[inline]
    pub fn get_arc(&self) -> &Arc<RegionData> {
        &self.0
    }
    /// Get the `ix`th parameter of this `Region`. Return an error on index out of bounds.
    #[inline]
    pub fn param(self, ix: usize) -> Result<Parameter, ()> {
        Parameter::try_new(self, ix)
    }
    /// Get the data behind this `Region`, if any
    #[inline]
    pub fn data(&self) -> &RegionData {
        &self.0
    }
    /// Get a pointer to the data behind this `Region`, or null if there is none
    #[inline]
    pub fn data_ptr(&self) -> *const RegionData {
        self.data() as *const _
    }
    /// Check whether this `Region` has any parameters
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data().is_empty()
    }
    /// Get the number of parameters of this `Region`
    #[inline]
    pub fn len(&self) -> usize {
        self.data().len()
    }
    /// Get the depth of this region
    #[inline]
    pub fn depth(&self) -> usize {
        self.data().depth()
    }
    /// Get the parent of this region if any
    #[inline]
    pub fn parent(&self) -> Option<&Region> {
        self.data().parent()
    }
    /// Iterate over the parameters of this `Region`.
    #[inline]
    pub fn params(self) -> impl Iterator<Item = Parameter> + ExactSizeIterator {
        let l = self.len();
        (0..l).map(move |ix| self.clone().param(ix).expect("Index always valid"))
    }
    /// Iterate over the parameters of this `Region`, borrowing a reference
    #[inline]
    pub fn borrow_params(&self) -> impl '_ + Iterator<Item = Parameter> {
        let l = self.len();
        (0..l).map(move |ix| self.clone().param(ix).expect("Index always valid"))
    }
    /// Get the conjunction of two regions, if any
    #[inline]
    pub fn conj<'a>(
        this: &'a Option<Region>,
        other: &'a Option<Region>,
    ) -> Result<&'a Option<Region>, Error> {
        use Ordering::*;
        match this.partial_cmp(other) {
            None => Err(Error::IncomparableRegions),
            Some(Greater) | Some(Equal) => Ok(this),
            Some(Less) => Ok(other),
        }
    }
}

impl Index<usize> for Region {
    type Output = TypeId;
    #[inline]
    fn index(&self, ix: usize) -> &TypeId {
        self.data().index(ix)
    }
}

impl Hash for Region {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.data_ptr(), hasher)
    }
}

impl<'a> RegionBorrow<'a> {
    /// Clone this region. This bumps the refcount
    #[inline]
    pub fn clone_region(&self) -> Region {
        Region(self.0.clone_arc())
    }
    /// Get the underlying `ArcBorrow` of this `RegionData`, if any
    pub fn get_borrow(&self) -> ArcBorrow<'a, RegionData> {
        self.0
    }
    /// Get the data behind this `Region`, if any
    #[inline]
    pub fn data(&self) -> &'a RegionData {
        self.0.get()
    }
    /// Get a pointer to the data behind this `Region`, or null if there is none
    #[inline]
    pub fn data_ptr(&self) -> *const RegionData {
        self.data() as *const _
    }
    /// Check whether this `Region` has any parameters
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data().is_empty()
    }
    /// Get the number of parameters of this `Region`
    #[inline]
    pub fn len(&self) -> usize {
        self.data().len()
    }
    /// Get the depth of this region
    #[inline]
    pub fn depth(&self) -> usize {
        self.data().depth()
    }
    /// Get the parent of this region if any
    #[inline]
    pub fn parent(&self) -> Option<&'a Region> {
        self.data().parent()
    }
    /// Get the `ix`th parameter of this `Region`. Return an error on index out of bounds.
    #[inline]
    pub fn param(self, ix: usize) -> Result<Parameter, ()> {
        self.clone_region().param(ix)
    }
    /// Get this region borrow as a region
    #[inline]
    pub fn as_region(&self) -> &Region {
        unsafe { &*(self as *const _ as *const Region) }
    }
}

impl Index<usize> for RegionBorrow<'_> {
    type Output = TypeId;
    #[inline]
    fn index(&self, ix: usize) -> &TypeId {
        self.data().index(ix)
    }
}

impl Hash for RegionBorrow<'_> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.data_ptr(), hasher)
    }
}