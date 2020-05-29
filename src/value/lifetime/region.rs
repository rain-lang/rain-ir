/*!
`rain` value regions
*/

use super::{LifetimeBorrow, Live};
use crate::util::hash_cache::Cache;
use crate::value::TypeId;
use crate::quick_pretty;
use lazy_static::lazy_static;
use smallvec::SmallVec;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use triomphe::{Arc, ArcBorrow};

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
#[derive(Debug, Clone, Eq)]
pub struct Region(Arc<RegionData>);

impl Region {
    /// Create a new reference from a given `RegionData`, caching if possible
    #[inline]
    pub fn new(data: RegionData) -> Region {
        Region(REGION_CACHE.cache(data))
    }
    /// Get a reference to a borrow of this region. More efficient than taking an `&Region`.
    #[inline]
    pub fn borrow_region(&self) -> RegionBorrow {
        RegionBorrow(self.0.borrow_arc())
    }
    /// Get the underlying `Arc` of this `Region`.
    #[inline]
    pub fn get_arc(&self) -> &Arc<RegionData> {
        &self.0
    }
}

impl Deref for Region {
    type Target = RegionData;
    #[inline]
    fn deref(&self) -> &RegionData {
        self.0.deref()
    }
}

impl PartialEq for Region {
    fn eq(&self, other: &Region) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Hash for Region {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.deref(), hasher)
    }
}

/// A borrow of a `rain` region
#[derive(Debug, Copy, Clone, Eq)]
pub struct RegionBorrow<'a>(ArcBorrow<'a, RegionData>);

impl<'a> RegionBorrow<'a> {
    /// Like `deref`, but using the lifetime of the `RegionBorrow` (which is incompatible with the `Deref` trait).
    #[inline]
    pub fn get(&self) -> &'a RegionData {
        self.0.get()
    }
    /// Clone this region. This bumps the refcount
    #[inline]
    pub fn clone_region(&self) -> Region {
        Region(self.0.clone_arc())
    }
    /// Get the underlying `ArcBorrow` of this `RegionData`
    pub fn get_borrow(&self) -> ArcBorrow<'a, RegionData> {
        self.0
    }
}

impl Deref for RegionBorrow<'_> {
    type Target = RegionData;
    #[inline]
    fn deref(&self) -> &RegionData {
        self.0.deref()
    }
}

impl PartialEq for RegionBorrow<'_> {
    fn eq(&self, other: &RegionBorrow) -> bool {
        ArcBorrow::ptr_eq(&self.0, &other.0)
    }
}

impl PartialEq<Region> for RegionBorrow<'_> {
    fn eq(&self, other: &Region) -> bool {
        ArcBorrow::ptr_eq(&self.0, &other.0.borrow_arc())
    }
}

impl Hash for RegionBorrow<'_> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.deref(), hasher)
    }
}

/// A vector of parameter types
pub type ParamTyVec = SmallVec<[TypeId; SMALL_PARAMS]>;

/// The data composing a `rain` region
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RegionData {
    /// The parent of this region
    parent: Option<Region>,
    /// The parameter types of this region
    param_tys: ParamTyVec,
    /// The depth of this region above the null region
    depth: usize,
}

impl Deref for RegionData {
    type Target = ParamTyVec;
    #[inline]
    fn deref(&self) -> &ParamTyVec {
        &self.param_tys
    }
}

impl DerefMut for RegionData {
    #[inline]
    fn deref_mut(&mut self) -> &mut ParamTyVec {
        &mut self.param_tys
    }
}

impl RegionData {
    /// Create data for a new region with a given parameter type vector and an optional parent region.
    #[inline]
    pub fn with(param_tys: ParamTyVec, parent: Option<Region>) -> RegionData {
        let depth = parent.as_ref().map(|parent| parent.depth).unwrap_or(0) + 1;
        RegionData {
            param_tys,
            parent,
            depth,
        }
    }
    /// Create data for a new, empty region with an optional parent region
    #[inline]
    pub fn new(parent: Option<Region>) -> RegionData {
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
        self.parent.as_ref()
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

impl Parameter {
    /**
     Reference the `ix`th parameter of the given region. Return `Err` if the parameter is out of bounds.

    # Examples
    Trying to make a parameter out of bounds returns `Err`:
    ```rust
    use rain_lang::value::lifetime::{Region, RegionData, Parameter};
    let empty_region = Region::new(RegionData::new(None));
    assert_eq!(Parameter::new(empty_region, 1), Err(()));
    ```
    */
    #[inline]
    pub fn new(region: Region, ix: usize) -> Result<Parameter, ()> {
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