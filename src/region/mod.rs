/*!
`rain` value regions
*/

use crate::eval::Apply;
use crate::lifetime::{LifetimeBorrow, Live};
use crate::typing::{Type, Typed};
use crate::util::hash_cache::Cache;
use crate::value::{TypeId, TypeRef, ValId, Value};
use crate::{quick_pretty, trivial_substitute};
use lazy_static::lazy_static;
use smallvec::SmallVec;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use triomphe::{Arc, ArcBorrow};

mod parametrized;
pub use parametrized::*;

/// The size of a small set of parameters to a `rain` region
pub const SMALL_PARAMS: usize = 2;

lazy_static! {
    /// The global cache of constructed regions.
    ///
    /// Note: region caching is not actually necessary for correctness, so consider exponsing a constructor
    /// for `Region`/`RegionBorrow` from `Arc<RegionData>` and `Arc<Region>`...
    pub static ref REGION_CACHE: Cache<RegionData> = Cache::new();
}

/// A `rain` region
#[derive(Debug, Clone, Eq, Default)]
pub struct Region(Option<Arc<RegionData>>);

impl Region {
    /// Create a new reference from a given `RegionData`, caching if possible
    #[inline]
    pub fn new(data: RegionData) -> Region {
        if data.is_null() {
            Region(None)
        } else {
            Region(Some(REGION_CACHE.cache(data)))
        }
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
    pub fn params(self) -> impl Iterator<Item = Parameter> {
        let l = self.len();
        (0..l).map(move |ix| self.clone().param(ix).expect("Index always valid"))
    }
    /// Iterate over the parameters of this `Region`, borrowing a reference
    #[inline]
    pub fn borrow_params(&self) -> impl '_ + Iterator<Item = Parameter> {
        let l = self.len();
        (0..l).map(move |ix| self.clone().param(ix).expect("Index always valid"))
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

/// A borrow of a `rain` region
#[derive(Debug, Copy, Clone, Eq, Default)]
pub struct RegionBorrow<'a>(Option<ArcBorrow<'a, RegionData>>);

impl<'a> RegionBorrow<'a> {
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
}

impl Deref for RegionBorrow<'_> {
    type Target = RegionData;
    #[inline]
    fn deref(&self) -> &RegionData {
        self.0.as_ref().map(|r| r.deref()).unwrap_or(&NULL_REGION)
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

/// A vector of parameter types
pub type ParamTyVec = SmallVec<[TypeId; SMALL_PARAMS]>;

/// The null region
pub static NULL_REGION: RegionData = RegionData {
    parent: Region(None),
    param_tys: None,
    depth: 0,
};

/// The data composing a `rain` region
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RegionData {
    /// The parent of this region
    parent: Region,
    /// The parameter types of this region
    param_tys: Option<ParamTyVec>,
    /// The depth of this region above the null region
    depth: usize,
}

impl Deref for RegionData {
    type Target = [TypeId];
    #[inline]
    fn deref(&self) -> &[TypeId] {
        if let Some(params) = &self.param_tys {
            params
        } else {
            &[]
        }
    }
}

impl DerefMut for RegionData {
    #[inline]
    fn deref_mut(&mut self) -> &mut [TypeId] {
        if let Some(params) = &mut self.param_tys {
            params
        } else {
            &mut []
        }
    }
}

impl RegionData {
    /// Create data for a new region with a given parameter type vector and a parent region
    #[inline]
    pub fn with(param_tys: ParamTyVec, parent: Region) -> RegionData {
        let depth = parent.depth + 1;
        RegionData {
            param_tys: Some(param_tys),
            parent,
            depth,
        }
    }
    /// Create data for a new, empty region with an optional parent region
    #[inline]
    pub fn new(parent: Region) -> RegionData {
        Self::with(SmallVec::new(), parent)
    }
    /// Get the depth of this region
    #[inline]
    pub fn depth(&self) -> usize {
        self.depth
    }
    /// Get the parent of this region
    #[inline]
    pub fn parent(&self) -> Option<&Region> {
        if self.is_null() {
            None
        } else {
            Some(&self.parent)
        }
    }
    /// Check if this region is the null region
    #[inline]
    pub fn is_null(&self) -> bool {
        self.depth == 0
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
        use Ordering::*;
        match self.depth.cmp(&other.depth) {
            Less => {
                if self < other {
                    Some(Less)
                } else {
                    None
                }
            }
            Equal => {
                if self == other {
                    Some(Equal)
                } else {
                    None
                }
            }
            Greater => {
                if self > other {
                    Some(Greater)
                } else {
                    None
                }
            }
        }
    }
    /**
    Check whether the right region is a parent of the left region.
    */
    #[inline]
    fn lt(&self, other: &RegionData) -> bool {
        if self.depth >= other.depth {
            return false;
        }
        let mut other_p = other.parent().expect("Impossible: self.depth < other.depth implies other.depth >= 2, i.e. other has a parent");
        while other_p.depth > self.depth {
            other_p = other.parent().expect("Impossible: self.depth < other_p.depth implies other.depth >= 2, i.e. other has a parent");
        }
        other_p.deref() == self
    }
    /**
    Check whether the left region is a parent of the right region
    */
    #[inline]
    fn gt(&self, other: &RegionData) -> bool {
        other.lt(self)
    }
    /**
    Check whether the left and right regions are equal, or the left is a parent of the right
    */
    #[inline]
    fn le(&self, other: &RegionData) -> bool {
        self == other || self.lt(other)
    }
    /**
    Check whether the left and right regions are equal, or the right is a parent of the left
    */
    #[inline]
    fn ge(&self, other: &RegionData) -> bool {
        other.le(self)
    }
}

/**
A parameter to a `rain` region.

Note that the uniqueness of `ValId`s corresponding to a given parameter is enforced by the hash-consing algorithm.
*/
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Parameter {
    /// The region this is a parameter for
    region: Region,
    /// The index of this parameter in the region's type vector
    ix: usize,
}

quick_pretty!(Parameter, s, fmt => write!(fmt, "#parameter(depth={}, ix={})", s.region().depth(), s.ix()));
trivial_substitute!(Parameter);

impl Parameter {
    /**
     Reference the `ix`th parameter of the given region. Return `Err` if the parameter is out of bounds.

    # Examples
    Trying to make a parameter out of bounds returns `Err`:
    ```rust
    use rain_lang::region::{Region, RegionData, Parameter};
    let empty_region = Region::new(RegionData::new(Region::default()));
    assert_eq!(Parameter::try_new(empty_region, 1), Err(()));
    ```
    */
    #[inline]
    pub fn try_new(region: Region, ix: usize) -> Result<Parameter, ()> {
        if ix >= region.len() {
            Err(())
        } else {
            Ok(Parameter { region, ix })
        }
    }
    /// Get the index of this parameter
    #[inline]
    pub fn ix(&self) -> usize {
        self.ix
    }
    /// Get this parameter's region
    #[inline]
    pub fn region(&self) -> &Region {
        &self.region
    }
}

impl Live for Parameter {
    fn lifetime(&self) -> LifetimeBorrow {
        self.region().borrow_region().into()
    }
}

impl Typed for Parameter {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.region[self.ix].borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        self.ty().is_universe()
    }
}

impl Apply for Parameter {}

impl Value for Parameter {
    fn no_deps(&self) -> usize {
        0
    }
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!("Attempted to get dependency {} of parameter #{} of a region, but parameters have no deps!", ix, self.ix)
    }
}
