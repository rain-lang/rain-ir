/*!
`rain` value regions
*/
use crate::value::{arr::TyArr, Error, TypeId};
use dashcache::{DashCache, GlobalCache};
use elysees::{Arc, ArcBorrow};
use im::{vector, Vector};
use lazy_static::lazy_static;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

mod parametrized;
pub use parametrized::*;
mod parameter;
pub use parameter::*;

/// A `rain` region
#[derive(Debug, Clone, Eq, Default)]
pub struct Region(Option<Arc<RegionData>>);

/// A borrow of a `rain` region
#[derive(Debug, Copy, Clone, Eq, Default)]
pub struct RegionBorrow<'a>(Option<ArcBorrow<'a, RegionData>>);

/// The data composing a `rain` region
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RegionData {
    /// The parents of this region
    parents: Option<Vector<Region>>,
    /// The parameter types of this region
    param_tys: TyArr,
}

/// A trait for objects which have a region
pub trait Regional {
    /// Get the region of this object
    #[inline]
    fn region(&self) -> RegionBorrow {
        RegionBorrow::default()
    }
    /// Get the depth of this object's region
    #[inline]
    fn depth(&self) -> usize {
        self.region().depth()
    }
}

/// The null region, with a given unique address
pub static NULL_REGION: RegionData = RegionData::NULL;

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
        if data.is_null() {
            Region(None)
        } else {
            Region(Some(REGION_CACHE.cache(data)))
        }
    }
    /// Create a new region with a given parameter type vector and a parent region
    #[inline]
    pub fn with(param_tys: TyArr, parent: Region) -> Region {
        Region::new(RegionData::with(param_tys, parent))
    }
    /// Create a new region with a given parent region and no parameters
    #[inline]
    pub fn with_parent(parent: Region) -> Region {
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
    /// Get the ancestor of this region up to a given depth
    #[inline]
    pub fn ancestor(&self, depth: usize) -> &Region {
        let mut ptr: &Region = self;
        while ptr.depth() > depth {
            ptr = ptr
                .parent()
                .expect("Depth > 0, so region must have a parent");
        }
        ptr
    }
    /// Get the conjunction of two regions, if any
    #[inline]
    pub fn conj<'a>(&'a self, other: &'a Region) -> Result<&'a Region, Error> {
        use Ordering::*;
        match self.partial_cmp(other) {
            None => Err(Error::IncomparableRegions),
            Some(Less) | Some(Equal) => Ok(self),
            Some(Greater) => Ok(other),
        }
    }
}

impl Deref for Region {
    type Target = RegionData;
    #[inline]
    fn deref(&self) -> &RegionData {
        if let Some(region) = &self.0 {
            region.deref()
        } else {
            &NULL_REGION
        }
    }
}

impl PartialEq for Region {
    fn eq(&self, other: &Region) -> bool {
        let self_ptr = self.deref() as *const _;
        let other_ptr = other.deref() as *const _;
        self_ptr == other_ptr
    }
}

impl PartialEq<RegionBorrow<'_>> for Region {
    fn eq(&self, other: &RegionBorrow) -> bool {
        let self_ptr = self.deref() as *const _;
        let other_ptr = other.deref() as *const _;
        self_ptr == other_ptr
    }
}

impl PartialEq<RegionData> for Region {
    fn eq(&self, other: &RegionData) -> bool {
        self.deref() == other
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
        self.deref().partial_cmp(other.deref())
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
        std::ptr::hash(self.deref(), hasher)
    }
}

impl<'a> RegionBorrow<'a> {
    /// A borrow of the null region
    pub const NULL: RegionBorrow<'a> = RegionBorrow(None);
    /// Like `deref`, but using the lifetime of the `RegionBorrow` (which is incompatible with the `Deref` trait).
    #[inline]
    pub fn get(&self) -> &'a RegionData {
        if let Some(region) = self.0 {
            region.get()
        } else {
            &NULL_REGION
        }
    }
    /// Clone this region. This bumps the refcount
    #[inline]
    pub fn clone_region(&self) -> Region {
        Region(self.0.map(|r| r.clone_arc()))
    }
    /// Get the underlying `ArcBorrow` of this `RegionData`, if any
    pub fn get_borrow(&self) -> Option<ArcBorrow<'a, RegionData>> {
        self.0
    }
    /// Get the ancestor of this region up to a given depth
    #[inline]
    pub fn ancestor(&self, depth: usize) -> RegionBorrow {
        let mut ptr: RegionBorrow = *self;
        while ptr.depth() > depth {
            ptr = ptr
                .get()
                .parent()
                .expect("Depth > 0, so region must have a parent")
                .borrow_region();
        }
        ptr
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

impl Deref for RegionBorrow<'_> {
    type Target = RegionData;
    #[inline]
    fn deref(&self) -> &RegionData {
        self.0.as_deref().unwrap_or(&NULL_REGION)
    }
}

impl PartialEq for RegionBorrow<'_> {
    fn eq(&self, other: &RegionBorrow) -> bool {
        let self_ptr = self.deref() as *const _;
        let other_ptr = other.deref() as *const _;
        self_ptr == other_ptr
    }
}

impl PartialEq<Region> for RegionBorrow<'_> {
    fn eq(&self, other: &Region) -> bool {
        let self_ptr = self.deref() as *const _;
        let other_ptr = other.deref() as *const _;
        self_ptr == other_ptr
    }
}

impl PartialEq<RegionData> for RegionBorrow<'_> {
    fn eq(&self, other: &RegionData) -> bool {
        //TODO: pointer check?
        self.deref() == other
    }
}

impl Hash for RegionBorrow<'_> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.deref(), hasher)
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
        self.deref().partial_cmp(other.deref())
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
        self.deref().partial_cmp(other)
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
        self.deref().partial_cmp(other)
    }
}

impl Deref for RegionData {
    type Target = [TypeId];
    #[inline]
    fn deref(&self) -> &[TypeId] {
        self.param_tys.deref()
    }
}

impl RegionData {
    /// The null region data
    pub const NULL: RegionData = RegionData {
        parents: None,
        param_tys: TyArr::EMPTY,
    };
    /// Create data for a new region with a given parameter type vector and a parent region
    #[inline]
    pub fn with(param_tys: TyArr, parent: Region) -> RegionData {
        let parents = if let Some(parents) = &parent.parents {
            let mut result = parents.clone();
            result.push_back(parent);
            result
        } else {
            vector![parent]
        };
        RegionData {
            param_tys,
            parents: Some(parents),
        }
    }
    /// Create data for a new, empty region with an optional parent region
    #[inline]
    pub fn with_parent(parent: Region) -> RegionData {
        Self::with(TyArr::default(), parent)
    }
    /// Get the depth of this region
    #[inline]
    pub fn depth(&self) -> usize {
        if let Some(parents) = &self.parents {
            parents.len()
        } else {
            0
        }
    }
    /// Get the parent of this region
    #[inline]
    pub fn parent(&self) -> Option<&Region> {
        if let Some(parents) = &self.parents {
            parents.last()
        } else {
            None
        }
    }
    /// Get the parameter types of this region
    #[inline]
    pub fn param_tys(&self) -> &TyArr {
        &self.param_tys
    }
    /// Check if this region is the null region
    #[inline]
    pub fn is_null(&self) -> bool {
        self.depth() == 0
    }
}

impl PartialOrd for RegionData {
    /**
    We define a region to be a subregion of another region if every value in one region lies in the other,
    which is true if and only if one of the regions is a parent of another. This naturally induces a partial
    ordering on the set of regions.
    */
    #[inline]
    fn partial_cmp(&self, other: &RegionData) -> Option<Ordering> {
        let left_depth = self.depth();
        let right_depth = other.depth();
        let min_depth = left_depth.min(right_depth);
        if min_depth == 0
            || self.parents.as_ref().unwrap()[min_depth - 1]
                == other.parents.as_ref().unwrap()[min_depth - 1]
        {
            Some(left_depth.cmp(&right_depth))
        } else {
            None
        }
    }
}
