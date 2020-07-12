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

/// A `rain` region
#[derive(Debug, Clone, Eq, Default)]
pub struct Region(Option<Arc<RegionData>>);

/// A borrow of a `rain` region
#[derive(Debug, Copy, Clone, Eq, Default)]
pub struct RegionBorrow<'a>(Option<ArcBorrow<'a, RegionData>>);

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
            let data = region.data().unwrap();
            Some(data.parents.get(depth).map(|region| region.borrow_region()).unwrap_or(region))
        } else {
            None
        }
    }
}

impl Regional for Option<Region> {
    #[inline]
    fn region(&self) -> Option<RegionBorrow> {
        None
    }
    /// Get the depth of this object's region
    #[inline]
    fn depth(&self) -> usize {
        0
    }
}

impl Regional for Option<RegionBorrow<'_>> {
    #[inline]
    fn region(&self) -> Option<RegionBorrow> {
        None
    }
    /// Get the depth of this object's region
    #[inline]
    fn depth(&self) -> usize {
        0
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
    /// The null region
    pub const NULL: Region = Region(None);
    /// Create a new reference from a given `RegionData`, caching if possible
    #[inline]
    pub fn new(data: RegionData) -> Region {
        Region(Some(REGION_CACHE.cache(data)))
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
        RegionBorrow(self.0.as_ref().map(|v| v.borrow_arc()))
    }
    /// Get the underlying `Arc` of this `Region`, if any
    #[inline]
    pub fn get_arc(&self) -> Option<&Arc<RegionData>> {
        self.0.as_ref()
    }
    /// Get the `ix`th parameter of this `Region`. Return an error on index out of bounds.
    #[inline]
    pub fn param(self, ix: usize) -> Result<Parameter, ()> {
        Parameter::try_new(self, ix)
    }
    /// Get the data behind this `Region`, if any
    #[inline]
    pub fn data(&self) -> Option<&RegionData> {
        self.0.as_deref()
    }
    /// Get a pointer to the data behind this `Region`, or null if there is none
    #[inline]
    pub fn data_ptr(&self) -> *const RegionData {
        self.0
            .as_deref()
            .map(|ptr| ptr as *const _)
            .unwrap_or(std::ptr::null())
    }
    /// Check whether this `Region` has any parameters
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data().map(|data| data.is_empty()).unwrap_or(false)
    }
    /// Get the number of parameters of this `Region`
    #[inline]
    pub fn len(&self) -> usize {
        self.data().map(|data| data.len()).unwrap_or(0)
    }
    /// Get the depth of this region
    #[inline]
    pub fn depth(&self) -> usize {
        self.data().map(|data| data.depth()).unwrap_or(0)
    }
    /// Get the parent of this region if any
    #[inline]
    pub fn parent(&self) -> Option<&Region> {
        self.0.as_ref().map(|data| data.parent()).flatten()
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
            Some(Less) | Some(Equal) => Ok(this),
            Some(Greater) => Ok(other),
        }
    }
}

impl Index<usize> for Region {
    type Output = TypeId;
    #[inline]
    fn index(&self, ix: usize) -> &TypeId {
        self.data().unwrap().index(ix)
    }
}

impl PartialEq for Region {
    fn eq(&self, other: &Region) -> bool {
        let self_ptr = self.data_ptr();
        let other_ptr = other.data_ptr();
        self_ptr == other_ptr
    }
}

impl PartialEq<RegionBorrow<'_>> for Region {
    fn eq(&self, other: &RegionBorrow) -> bool {
        let self_ptr = self.data_ptr();
        let other_ptr = other.data_ptr();
        self_ptr == other_ptr
    }
}

impl PartialEq<RegionData> for Region {
    fn eq(&self, other: &RegionData) -> bool {
        self.data() == Some(other)
    }
}

impl PartialOrd for Region {
    /**
    We define a region to be a subregion of another region if every value in one region lies in the other,
    which is true if and only if one of the regions is a parent of another. This naturally induces a partial
    ordering on the set of regions.
    */
    #[inline]
    fn partial_cmp(&self, other: &Region) -> Option<Ordering> {
        self.data().partial_cmp(&other.data())
    }
}

impl PartialOrd<RegionData> for Region {
    /**
    We define a region to be a subregion of another region if every value in one region lies in the other,
    which is true if and only if one of the regions is a parent of another. This naturally induces a partial
    ordering on the set of regions.
    */
    #[inline]
    fn partial_cmp(&self, other: &RegionData) -> Option<Ordering> {
        self.deref().partial_cmp(other)
    }
}

impl PartialOrd<RegionBorrow<'_>> for Region {
    /**
    We define a region to be a subregion of another region if every value in one region lies in the other,
    which is true if and only if one of the regions is a parent of another. This naturally induces a partial
    ordering on the set of regions.
    */
    #[inline]
    fn partial_cmp(&self, other: &RegionBorrow) -> Option<Ordering> {
        self.deref().partial_cmp(other)
    }
}

impl Hash for Region {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.data_ptr(), hasher)
    }
}

impl<'a> RegionBorrow<'a> {
    /// A borrow of the null region
    pub const NULL: RegionBorrow<'a> = RegionBorrow(None);
    /// Clone this region. This bumps the refcount
    #[inline]
    pub fn clone_region(&self) -> Region {
        Region(self.0.map(|r| r.clone_arc()))
    }
    /// Get the underlying `ArcBorrow` of this `RegionData`, if any
    pub fn get_borrow(&self) -> Option<ArcBorrow<'a, RegionData>> {
        self.0
    }
    /// Get the data behind this `Region`, if any
    #[inline]
    pub fn data(&self) -> Option<&'a RegionData> {
        self.0.map(|borrow| borrow.get())
    }
    /// Get a pointer to the data behind this `Region`, or null if there is none
    #[inline]
    pub fn data_ptr(&self) -> *const RegionData {
        self.0
            .as_deref()
            .map(|ptr| ptr as *const _)
            .unwrap_or(std::ptr::null())
    }
    /// Check whether this `Region` has any parameters
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data().map(|data| data.is_empty()).unwrap_or(false)
    }
    /// Get the number of parameters of this `Region`
    #[inline]
    pub fn len(&self) -> usize {
        self.data().map(|data| data.len()).unwrap_or(0)
    }
    /// Get the depth of this region
    #[inline]
    pub fn depth(&self) -> usize {
        self.data().map(|data| data.depth()).unwrap_or(0)
    }
    /// Get the parent of this region if any
    #[inline]
    pub fn parent(&self) -> Option<&'a Region> {
        self.data().map(|data| data.parent()).flatten()
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

impl PartialEq for RegionBorrow<'_> {
    fn eq(&self, other: &RegionBorrow) -> bool {
        let self_ptr = self.data_ptr();
        let other_ptr = other.data_ptr();
        self_ptr == other_ptr
    }
}

impl PartialEq<Region> for RegionBorrow<'_> {
    fn eq(&self, other: &Region) -> bool {
        let self_ptr = self.data_ptr();
        let other_ptr = other.data_ptr();
        self_ptr == other_ptr
    }
}

impl PartialEq<RegionData> for RegionBorrow<'_> {
    fn eq(&self, other: &RegionData) -> bool {
        //TODO: pointer check?
        self.data() == Some(other)
    }
}

impl Hash for RegionBorrow<'_> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.data_ptr(), hasher)
    }
}

impl PartialOrd for RegionBorrow<'_> {
    /**
    We define a region to be a subregion of another region if every value in one region lies in the other,
    which is true if and only if one of the regions is a parent of another. This naturally induces a partial
    ordering on the set of regions.
    */
    #[inline]
    fn partial_cmp(&self, other: &RegionBorrow<'_>) -> Option<Ordering> {
        self.data().partial_cmp(&other.data())
    }
}

impl PartialOrd<RegionData> for RegionBorrow<'_> {
    /**
    We define a region to be a subregion of another region if every value in one region lies in the other,
    which is true if and only if one of the regions is a parent of another. This naturally induces a partial
    ordering on the set of regions.
    */
    #[inline]
    fn partial_cmp(&self, other: &RegionData) -> Option<Ordering> {
        self.data().partial_cmp(&Some(other))
    }
}

impl PartialOrd<Region> for RegionBorrow<'_> {
    /**
    We define a region to be a subregion of another region if every value in one region lies in the other,
    which is true if and only if one of the regions is a parent of another. This naturally induces a partial
    ordering on the set of regions.
    */
    #[inline]
    fn partial_cmp(&self, other: &Region) -> Option<Ordering> {
        self.data().partial_cmp(&other.data())
    }
}
