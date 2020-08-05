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
/// 
/// Regions are one of the central constructs in `rain`, and are, in essence, our take on the
/// "Regionalized" in "Regionalized Value-State Dependence Graph" (RVSDG). Every `rain` value is
/// assigned exactly one region, and, as a rule of thumb, values may depend only on values in the
/// same region or an ancestor of that region. All regions have the null region as ancestors, in
/// which only constants reside.
/// 
/// As of now, regions are defined by a set of parameters coupled with a parent. Various extensions
/// are planned to this, including, but not limited to
/// - Termination typing, for potentially non-terminating nodes such as phi-nodes
/// - Monadic regions
#[derive(Debug, Clone, Eq)]
pub struct Region(Arc<RegionData>);

/// A borrow of a `rain` region
/// 
/// This, in essence, serves as a more efficient version of [`&Region`](Region): 
/// since [`Region`](Region) is `Arc`-backed, an [`&Region`](Region) requires two
/// pointer dereferences to access the underlying [`RegionData`](RegionData). This
/// struct, on the other hand, stores a direct pointer to the [`RegionData`](RegionData)
/// using `elysees`' `ArcBorrow`. This also removes the need to have a ]`Region`](Region) 
/// instance on the stack at all, which can be helpful in certain cases.
/// 
/// In cases where an [`&Region`](Region) is needed, a [`RegionBorrow`](RegionBorrow) can
/// be dereferenced into one with the [`as_region`](RegionBorrow::as_region) method.
#[derive(Debug, Copy, Clone, Eq)]
pub struct RegionBorrow<'a>(ArcBorrow<'a, RegionData>);

/// A trait for objects which lie in a region, or representations of regions
/// 
/// This trait is designed to both handle region representations polymorphically
/// (e.g., handling [`Region`](Region), [`Option<Region>`](Region) and [`RegionBorrow`](RegionBorrow) using the same function)
/// as well as to group objects (such as `rain` [`Value`](crate::value)s) which are known to lie in a single-region.
/// 
/// TODO: consider making `lcr` et. al. a method of `Regional`...
pub trait Regional {
    /// Get the region of this object
    ///
    /// Returns the region this object is in, or `None` if the object is in the null region
    /// Unlike [`cloned_region`](Regional::cloned_region), returns a borrowed [`RegionBorrow`](RegionBorrow) (instead of a [`Region`](Region)) on success.
    /// For correctness, this method should otherwise return the same result as [`cloned_region`](Regional::cloned_region).
    ///
    /// # Example
    /// ```rust
    /// use rain_ir::region::{Region, Regional};
    /// use rain_ir::primitive::logical::Bool;
    /// use rain_ir::typing::Type;
    ///
    /// // Constants reside in the null region:
    ///
    /// assert_eq!(true.region(), None);
    /// assert_eq!(false.region(), None);
    ///
    /// // Parameters reside in their region:
    ///
    /// // We construct the region of a function taking a single bool as a parameter
    /// let region = Region::with(std::iter::once(Bool.into_ty()).collect(), None);
    ///
    /// // We extract the first parameter
    /// let param = region.clone().param(0).unwrap();
    /// assert_eq!(param.region(), Some(region.borrow_region()));
    ///
    /// // Regions return themselves as a region
    /// assert_eq!(region.region(), Some(region.borrow_region()));
    ///
    /// // An `Option` works too
    /// let mut opt = Some(region.clone());
    /// assert_eq!(opt.region(), Some(region.borrow_region()));
    /// opt = None;
    /// assert_eq!(opt.region(), None);
    /// ```
    #[inline]
    fn region(&self) -> Option<RegionBorrow> {
        None
    }
    /// Get the region of this object, cloned
    ///
    /// Returns the region this object is in, or `None` if the object is in the null region.
    /// Unlike [`region`](Regional::region), returns an owned [`Region`](Region) (instead of a [`RegionBorrow`](RegionBorrow)) on success.
    /// For correctness, this method should otherwise return the same result as [`region`](Regional::region).    
    ///
    /// TODO: consider renaming to `clone_region`...
    ///
    /// # Example
    /// ```rust
    /// use rain_ir::region::{Region, Regional};
    /// use rain_ir::primitive::logical::Bool;
    /// use rain_ir::typing::Type;
    ///
    /// // Constants reside in the null region:
    ///
    /// assert_eq!(true.cloned_region(), None);
    /// assert_eq!(false.cloned_region(), None);
    ///
    /// // Parameters reside in their region:
    ///
    /// // We construct the region of a function taking a single bool as a parameter
    /// let region = Region::with(std::iter::once(Bool.into_ty()).collect(), None);
    ///
    /// // We extract the first parameter
    /// let param = region.clone().param(0).unwrap();
    /// assert_eq!(param.cloned_region(), Some(region.clone()));
    ///
    /// // Regions return themselves as a region
    /// assert_eq!(region.cloned_region(), Some(region.clone()));
    ///
    /// // An `Option` works too
    /// let mut opt = Some(region.clone());
    /// assert_eq!(opt.cloned_region(), Some(region));
    /// opt = None;
    /// assert_eq!(opt.cloned_region(), None);
    /// ```
    #[inline]
    fn cloned_region(&self) -> Option<Region> {
        self.region().map(|region| region.clone_region())
    }
    /// Get the depth of the region associated with this object
    ///
    /// The depth of a region is defined inductively as follows
    /// - The null region has depth `0`
    /// - A region has the depth of it's parent plus one
    /// For correctness, we must have `self.depth() == self.region.depth()`
    ///
    /// # Example
    /// ```rust
    /// use rain_ir::region::{Region, Regional};
    /// use rain_ir::primitive::logical::Bool;
    /// use rain_ir::typing::Type;
    ///
    /// // Constants reside in the null region:
    ///
    /// assert_eq!(true.depth(), 0);
    /// assert_eq!(false.depth(), 0);
    ///
    /// // Parameters reside in their region:
    ///
    /// // We construct the region of a function taking a single bool as a parameter
    /// let region = Region::with(std::iter::once(Bool.into_ty()).collect(), None);
    ///
    /// // We extract the first parameter
    /// let param = region.clone().param(0).unwrap();
    /// assert_eq!(param.depth(), 1);
    ///
    /// // We can, of course, call this function directly on a region
    /// assert_eq!(region.depth(), 1);
    ///
    /// // An `Option` works too, with `None` representing the null region
    /// let mut opt = Some(region.clone());
    /// assert_eq!(opt.depth(), 1);
    /// opt = None;
    /// assert_eq!(opt.depth(), 0);
    #[inline]
    fn depth(&self) -> usize {
        self.region().depth()
    }
    /// Get the ancestor of this region up to a given depth, or this value's region if `depth >= self.depth()`
    #[inline]
    fn ancestor(&self, depth: usize) -> Option<RegionBorrow> {
        if depth == 0 {
            return None;
        }
        if let Some(region) = self.region() {
            Some(
                region
                    .data()
                    .parents
                    .get(depth - 1)
                    .map(|region| region.borrow_region())
                    .unwrap_or(region),
            )
        } else {
            None
        }
    }
}

/// Get the smallest region containing two objects or regions
///
/// Returns the smallest region containing two objects or regions if such a region exists. If the regions of the
/// objects are incomparable, return `None`.
///
/// TODO: think about this behaviour: would returning `Err` be more appropriate?
pub fn lcr<'a, L: Regional, R: Regional>(left: &'a L, right: &'a R) -> Option<RegionBorrow<'a>> {
    use Ordering::*;
    let lr = left.region();
    let rr = right.region();
    match lr.partial_cmp(&rr) {
        Some(Less) | Some(Equal) => lr,
        Some(Greater) => rr,
        None => None,
    }
}

lazy_static! {
    /// The global cache of constructed regions.
    pub static ref REGION_CACHE: DashCache<Arc<RegionData>> = DashCache::new();
}

impl Region {
    /// Create a new reference from a given [`RegionData`](RegionData), caching if possible
    #[inline]
    pub fn new(data: RegionData) -> Region {
        Region(REGION_CACHE.cache(data))
    }
    /// Create a new region with a given parameter type vector and a parent region
    #[inline]
    pub fn with(param_tys: TyArr, parent: Option<Region>) -> Region {
        Region::new(RegionData::with(param_tys, parent))
    }
    /// Create a new region with a given parent region, having no parameters
    /// 
    /// This is a bit useless now, but eventually when termination typing comes around this may come in handy.
    #[inline]
    pub fn with_parent(parent: Option<Region>) -> Region {
        Region::new(RegionData::with_parent(parent))
    }
    /// Get a reference to a borrow of this region. More efficient than taking an [`&Region`](Region)
    #[inline]
    pub fn borrow_region(&self) -> RegionBorrow {
        RegionBorrow(self.0.borrow_arc())
    }
    /// Get the underlying `elysees::Arc` of this [`Region`](Region), if any
    /// 
    /// TODO: add an `into_arc` method?
    #[inline]
    pub fn get_arc(&self) -> &Arc<RegionData> {
        &self.0
    }
    /// Get the `ix`th parameter of this [`Region`](Region). Return an error on index out of bounds.
    #[inline]
    pub fn param(self, ix: usize) -> Result<Parameter, Error> {
        //TODO: `into_param` pattern
        Parameter::try_new(self, ix)
    }
    /// Get the data behind this [`Region`](Region)
    /// 
    /// This method will always return the same value as `self.deref()`
    #[inline]
    pub fn data(&self) -> &RegionData {
        &self.0
    }
    /// Get a pointer to the data behind this [`Region`](Region)
    /// 
    /// This method will always return the same value as `self.data() as *const _` or `self.deref() as *const _`.
    #[inline]
    pub fn data_ptr(&self) -> *const RegionData {
        self.data() as *const _
    }
    /// Check whether this [`Region`](Region) has any parameters
    ///
    /// This method will return `true` if and only if `self.len() == 0`
    /// 
    /// # Examples
    /// ```rust
    /// use rain_ir::region::Region;
    /// use rain_ir::primitive::logical::Bool;
    /// use rain_ir::typing::Type;
    /// 
    /// let empty_region = Region::with_parent(None);
    /// let nested_empty = Region::with_parent(Some(empty_region.clone()));
    /// let nested_full = Region::with(
    ///     std::iter::once(Bool.into_ty()).collect(), 
    ///     Some(nested_empty.clone())
    /// );
    /// 
    /// assert!(empty_region.is_empty());
    /// assert!(nested_empty.is_empty());
    /// assert!(!nested_full.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data().is_empty()
    }
    /// Get the number of parameters of this [`Region`](Region)
    /// 
    /// # Examples
    /// ```rust
    /// use rain_ir::region::Region;
    /// use rain_ir::primitive::logical::Bool;
    /// use rain_ir::typing::Type;
    /// 
    /// let empty_region = Region::with_parent(None);
    /// let nested_empty = Region::with_parent(Some(empty_region.clone()));
    /// let nested_full = Region::with(
    ///     std::iter::once(Bool.into_ty()).collect(), 
    ///     Some(nested_empty.clone())
    /// );
    /// let nested_many = Region::with(
    ///     vec![Bool.into_ty(), Bool.into_ty(), Bool.into_ty()].into_iter().collect(),
    ///     Some(empty_region.clone())
    /// );
    /// 
    /// assert_eq!(empty_region.len(), 0);
    /// assert_eq!(nested_empty.len(), 0);
    /// assert_eq!(nested_full.len(), 1);
    /// assert_eq!(nested_many.len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.data().len()
    }
    /// Get the parent of this region, if any
    #[inline]
    pub fn parent(&self) -> Option<&Region> {
        self.data().parent()
    }
    /// Iterate over the parameters of this [`Region`](Region).
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
    /// 
    /// The conjunction of two regions is defined to be the largest region contained in both
    /// regions. It is guaranteed, in the current design, to be one of the two regions if it
    /// exists. If two regions have no conjunction, this function returns a [`value::Error`](Error).
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
    /// Get the parent of this region if any
    #[inline]
    pub fn parent(&self) -> Option<&'a Region> {
        self.data().parent()
    }
    /// Get the `ix`th parameter of this `Region`. Return an error on index out of bounds.
    #[inline]
    pub fn param(self, ix: usize) -> Result<Parameter, Error> {
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
