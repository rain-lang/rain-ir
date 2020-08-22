/*!
`rain` value regions
*/
use crate::typing::primitive::PROP;
use crate::value::{arr::TyArr, Error, TypeId, UniverseId};
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
#[derive(Debug, Clone, Eq, Default)]
pub struct Region(Option<Arc<RegionData>>);

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
#[derive(Debug, Copy, Clone, Eq, Default)]
pub struct RegionBorrow<'a>(Option<ArcBorrow<'a, RegionData>>);

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
    /// use std::iter::once;
    /// use rain_ir::region::{Region, Regional};
    /// use rain_ir::primitive::logical::Bool;
    /// use rain_ir::typing::Type;
    ///
    /// // Constants reside in the null region:
    ///
    /// assert_eq!(true.region(), Region::NULL);
    /// assert_eq!(false.region(), Region::NULL);
    ///
    /// // Parameters reside in their region:
    ///
    /// // We construct the region of a function taking a single bool as a parameter
    /// let region = Region::with(once(Bool.into_ty()).collect(), Region::NULL).unwrap();
    ///
    /// // We extract the first parameter
    /// let param = region.clone().param(0).unwrap();
    /// assert_eq!(param.region(), region);
    ///
    /// // Regions return themselves as a region
    /// assert_eq!(region.region(), region);
    /// ```
    #[inline]
    fn region(&self) -> RegionBorrow {
        RegionBorrow::default()
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
    /// use std::iter::once;
    /// use rain_ir::region::{Region, Regional};
    /// use rain_ir::primitive::logical::Bool;
    /// use rain_ir::typing::Type;
    ///
    /// // Constants reside in the null region:
    ///
    /// assert_eq!(true.clone_region(), Region::NULL);
    /// assert_eq!(false.clone_region(), Region::NULL);
    ///
    /// // Parameters reside in their region:
    ///
    /// // We construct the region of a function taking a single bool as a parameter
    /// let region = Region::with(once(Bool.into_ty()).collect(), Region::NULL).unwrap();
    ///
    /// // We extract the first parameter
    /// let param = region.param(0).unwrap();
    /// assert_eq!(param.clone_region(), region);
    ///
    /// // Regions return themselves as a region
    /// assert_eq!(region.clone_region(), region);
    /// ```
    #[inline]
    fn clone_region(&self) -> Region {
        self.region().clone_region()
    }
    /// Get the greatest region containing this object and another, if any
    #[inline]
    fn gcr<'a, R: Regional>(&'a self, other: &'a R) -> Result<RegionBorrow<'a>, Error> {
        self.region().get_gcr(other.region())
    }
    /// Get the greatest common region containing this object and those in an iterator, if any
    #[inline]
    fn gcrs<'a, R, I>(&'a self, other: I) -> Result<RegionBorrow<'a>, Error>
    where
        R: Regional + 'a,
        I: Iterator<Item = &'a R>,
    {
        let mut gcr = self.region();
        for regional in other {
            gcr = gcr.get_gcr(regional.region())?;
        }
        Ok(gcr)
    }
    /// Get the least common region containing this object and another, if any
    #[inline]
    fn lcr<'a, R: Regional>(&'a self, other: &'a R) -> Result<RegionBorrow<'a>, Error> {
        self.region().get_lcr(other.region())
    }
    /// Get the least common region containing this object and those in an iterator, if any
    #[inline]
    fn lcrs<'a, R, I>(&'a self, other: I) -> Result<RegionBorrow<'a>, Error>
    where
        R: Regional + 'a,
        I: Iterator<Item = &'a R>,
    {
        let mut lcr = self.region();
        for regional in other {
            lcr = lcr.get_lcr(regional.region())?;
        }
        Ok(lcr)
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
    /// use std::iter::once;
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
    /// let region = Region::with(once(Bool.into_ty()).collect(), Region::NULL).unwrap();
    ///
    /// // We extract the first parameter
    /// let param = region.param(0).unwrap();
    /// assert_eq!(param.depth(), 1);
    ///
    /// // We can, of course, call this function directly on a region
    /// assert_eq!(region.depth(), 1);
    #[inline]
    fn depth(&self) -> usize {
        self.region().depth()
    }
    /// Get the ancestor of this region up to a given depth, or this value's region if `depth >= self.depth()`
    #[inline]
    fn ancestor(&self, depth: usize) -> RegionBorrow {
        if depth == 0 {
            return RegionBorrow::NULL;
        }
        let region = self.region();
        if let Some(data) = region.data() {
            data.parents
                .get(depth - 1)
                .map(|region| region.borrow_region())
                .unwrap_or(region)
        } else {
            //TODO: think about this, when termination flags are added to regions...
            region
        }
    }
}

lazy_static! {
    /// The global cache of constructed regions.
    pub static ref REGION_CACHE: DashCache<Arc<RegionData>> = DashCache::new();
}

impl Region {
    /// The null region
    pub const NULL: Region = Region(None);
    /// Create a new reference from a given [`RegionData`](RegionData), caching if possible
    #[inline]
    pub fn new(data: RegionData) -> Region {
        Region(Some(REGION_CACHE.cache(data)))
    }
    /// Create data for a new region with a given parameter type vector and a parent region
    ///
    /// This constructor does not check whether all parameter types lie within the given parent region, but it is a *logic error* if they do not!
    #[inline]
    pub fn with_unchecked(param_tys: TyArr, parent: Region, universe: UniverseId) -> Region {
        Region::new(RegionData::with_unchecked(param_tys, parent, universe))
    }
    /// Create a new region with a given parameter type vector and a parent region
    #[inline]
    pub fn with(param_tys: TyArr, parent: Region) -> Result<Region, Error> {
        RegionData::with(param_tys, parent).map(Region::new)
    }
    /// Get the minimal region for a set of parameters
    #[inline]
    pub fn minimal(param_tys: TyArr) -> Result<Region, Error> {
        RegionData::minimal(param_tys).map(Region::new)
    }
    /// Create a new region with a given parent region, having no parameters
    ///
    /// This is a bit useless now, but eventually when termination typing comes around this may come in handy.
    #[inline]
    pub fn with_parent(parent: Region) -> Region {
        Region::new(RegionData::with_parent(parent))
    }
    /// Get a reference to a borrow of this region. More efficient than taking an [`&Region`](Region)
    #[inline]
    pub fn borrow_region(&self) -> RegionBorrow {
        RegionBorrow(self.0.as_ref().map(Arc::borrow_arc))
    }
    /// Get the underlying `elysees::Arc` of this [`Region`](Region), if any
    ///
    /// TODO: add an `into_arc` method?
    #[inline]
    pub fn get_arc(&self) -> Option<&Arc<RegionData>> {
        self.0.as_ref()
    }
    /// Get the `ix`th parameter of this [`Region`](Region). Return an error on index out of bounds.
    #[inline]
    pub fn param(&self, ix: usize) -> Result<Parameter, Error> {
        Parameter::try_clone_new(self, ix)
    }
    /// Get the `ix`th parameter of this [`Region`](Region), consuming it. Return an error on index out of bounds.
    #[inline]
    pub fn into_param(self, ix: usize) -> Result<Parameter, Error> {
        Parameter::try_new(self, ix)
    }
    /// Get the universe of this region's parameters
    #[inline]
    pub fn universe(&self) -> &UniverseId {
        self.data()
            .map(|data| &data.universe)
            .unwrap_or_else(|| PROP.coerce_ref())
    }
    /// Get the data behind this [`Region`](Region), if any
    #[inline]
    pub fn data(&self) -> Option<&RegionData> {
        self.0.as_deref()
    }
    /// Get a pointer to the data behind this [`Region`](Region)
    #[inline]
    pub fn data_ptr(&self) -> *const RegionData {
        self.data()
            .map(|data| data as *const _)
            .unwrap_or(std::ptr::null())
    }
    /// Check whether this [`Region`](Region) has any parameters
    ///
    /// This method will return `true` if and only if `self.len() == 0`
    ///
    /// # Examples
    /// ```rust
    /// use std::iter::once;
    /// use rain_ir::region::Region;
    /// use rain_ir::primitive::logical::Bool;
    /// use rain_ir::typing::Type;
    ///
    /// let empty_region = Region::with_parent(Region::NULL);
    /// let nested_empty = Region::with_parent(empty_region.clone());
    /// let nested_full = Region::with(
    ///     once(Bool.into_ty()).collect(),
    ///     nested_empty.clone()
    /// ).unwrap();
    ///
    /// assert!(empty_region.is_empty());
    /// assert!(nested_empty.is_empty());
    /// assert!(!nested_full.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data().map(|data| data.is_empty()).unwrap_or(true)
    }
    ///  Check whether this [`Region`](Region) is the null region
    #[inline]
    pub fn is_null(&self) -> bool {
        self.0.is_none()
    }
    ///  Check whether this [`Region`](Region) is the not null region
    #[inline]
    pub fn is_nonnull(&self) -> bool {
        self.0.is_some()
    }
    /// Get the number of parameters of this [`Region`](Region)
    ///
    /// # Examples
    /// ```rust
    /// use rain_ir::region::Region;
    /// use rain_ir::primitive::logical::Bool;
    /// use rain_ir::typing::Type;
    ///
    /// let empty_region = Region::with_parent(Region::NULL);
    /// let nested_empty = Region::with_parent(empty_region.clone());
    /// let nested_full = Region::with(
    ///     std::iter::once(Bool.into_ty()).collect(),
    ///     nested_empty.clone()
    /// ).unwrap();
    /// let nested_many = Region::with(
    ///     vec![Bool.into_ty(), Bool.into_ty(), Bool.into_ty()].into_iter().collect(),
    ///     empty_region.clone()
    /// ).unwrap();
    ///
    /// assert_eq!(empty_region.len(), 0);
    /// assert_eq!(nested_empty.len(), 0);
    /// assert_eq!(nested_full.len(), 1);
    /// assert_eq!(nested_many.len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.data().map(|data| data.len()).unwrap_or(0)
    }
    /// Get the parent of this region, if any
    #[inline]
    pub fn parent(&self) -> &Region {
        self.data()
            .map(|data| data.parent())
            .unwrap_or(&Region::NULL)
    }
    /// Iterate over the parameters of this [`Region`](Region)
    #[inline]
    pub fn params(
        &self,
    ) -> impl Iterator<Item = Parameter> + ExactSizeIterator + DoubleEndedIterator {
        self.clone().into_params()
    }
    /// Iterate over the parameters of this [`Region`](Region), consuming it
    #[inline]
    pub fn into_params(
        self,
    ) -> impl Iterator<Item = Parameter> + ExactSizeIterator + DoubleEndedIterator {
        let l = self.len();
        (0..l).map(move |ix| self.clone().param(ix).expect("Index always valid"))
    }
    /// Get the parameter types of this region
    #[inline]
    pub fn param_tys(&self) -> &TyArr {
        static EMPTY: TyArr = TyArr::EMPTY;
        if let Some(data) = self.data() {
            data.param_tys()
        } else {
            &EMPTY
        }
    }
}

impl Index<usize> for Region {
    type Output = TypeId;
    #[inline]
    fn index(&self, ix: usize) -> &TypeId {
        self.data().expect("The null region is empty").index(ix)
    }
}

impl Hash for Region {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.data_ptr(), hasher)
    }
}

impl<'a> RegionBorrow<'a> {
    /// A borrow of the null region
    pub const NULL: RegionBorrow<'static> = RegionBorrow(None);
    /// Clone this region. This bumps the refcount
    #[inline]
    pub fn clone_region(&self) -> Region {
        Region(self.0.as_ref().map(ArcBorrow::clone_arc))
    }
    /// Get the underlying `ArcBorrow` of this `RegionData`, if any
    pub fn get_borrow(&self) -> Option<ArcBorrow<'a, RegionData>> {
        self.0
    }
    /// Get the data behind this `Region`, if any
    #[inline]
    pub fn data(&self) -> Option<&'a RegionData> {
        self.0.as_ref().map(ArcBorrow::get)
    }
    /// Get the parent of this region if any
    #[inline]
    pub fn parent(&self) -> &'a Region {
        self.data().map(RegionData::parent).unwrap_or(&Region::NULL)
    }
    /// Get this region borrow as a region
    #[inline]
    pub fn as_region(&self) -> &Region {
        unsafe { &*(self as *const _ as *const Region) }
    }
    /// Get the greatest region containing this object and another, if any
    #[inline]
    fn get_gcr(self, other: RegionBorrow<'a>) -> Result<RegionBorrow<'a>, Error> {
        match self.partial_cmp(&other) {
            Some(Ordering::Less) => Ok(other),
            Some(_) => Ok(self),
            _ => Err(Error::IncomparableRegions),
        }
    }
    /// Get the greatest region containing this object and another, if any
    #[inline]
    fn get_lcr(self, other: RegionBorrow<'a>) -> Result<RegionBorrow<'a>, Error> {
        match self.partial_cmp(&other) {
            Some(Ordering::Greater) => Ok(other),
            Some(_) => Ok(self),
            _ => Err(Error::IncomparableRegions),
        }
    }
}

impl Deref for RegionBorrow<'_> {
    type Target = Region;
    #[inline]
    fn deref(&self) -> &Region {
        self.as_region()
    }
}

impl Index<usize> for RegionBorrow<'_> {
    type Output = TypeId;
    #[inline]
    fn index(&self, ix: usize) -> &TypeId {
        self.data().expect("The null region is empty!").index(ix)
    }
}

impl Hash for RegionBorrow<'_> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.data_ptr(), hasher)
    }
}
