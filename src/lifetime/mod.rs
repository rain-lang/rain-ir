/*!
`rain` value lifetimes

`rain`'s lifetime system centers around the `Lifetime` object, which semantically represents a distinct `rain` lifetime.
`Lifetime` objects are automatically managed and garbage collected by a global lifetime cache. This module contains the
definitions for the `Lifetime` object, the lifetime cache, a variety of auxiliary objects (e.g. `LifetimeBorrow` to avoid
pointer-chasing) and implementations of a variety of algorithms used in the lifetime system.

*/
use crate::region::{Region, RegionBorrow, Regional};
use crate::value::Error;
use dashcache::{DashCache, GlobalCache};
use elysees::{Arc, ArcBorrow};
use lazy_static::lazy_static;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::ops::{BitAnd, Mul};

mod arr;
pub use arr::*;
mod color;
pub use color::*;
mod data;
pub use data::*;
mod affine;
pub use affine::*;
mod relevant;
pub use relevant::*;

lazy_static! {
    /// The global lifetime cache
    pub static ref LIFETIME_CACHE: DashCache<Arc<LifetimeData>> = DashCache::new();
}

/// A `rain` lifetime
#[derive(Debug, Clone, Eq, Default)]
#[repr(transparent)]
pub struct Lifetime(Option<Arc<LifetimeData>>);

impl PartialEq for Lifetime {
    fn eq(&self, other: &Lifetime) -> bool {
        let self_ptr = self.deref() as *const _;
        let other_ptr = other.deref() as *const _;
        self_ptr == other_ptr
    }
}

impl Hash for Lifetime {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.deref(), hasher)
    }
}

impl Deref for Lifetime {
    type Target = LifetimeData;
    fn deref(&self) -> &LifetimeData {
        if let Some(ptr) = &self.0 {
            &ptr
        } else {
            &STATIC_LIFETIME
        }
    }
}

/// A borrow of a `rain` lifetime
#[derive(Debug, Copy, Clone, Eq, Default)]
pub struct LifetimeBorrow<'a>(Option<ArcBorrow<'a, LifetimeData>>);

impl PartialEq for LifetimeBorrow<'_> {
    fn eq(&self, other: &LifetimeBorrow) -> bool {
        let self_ptr = self.deref() as *const _;
        let other_ptr = other.deref() as *const _;
        self_ptr == other_ptr
    }
}

impl PartialEq<Lifetime> for LifetimeBorrow<'_> {
    fn eq(&self, other: &Lifetime) -> bool {
        *self == other.borrow_lifetime()
    }
}

impl PartialEq<LifetimeBorrow<'_>> for Lifetime {
    fn eq(&self, other: &LifetimeBorrow) -> bool {
        self.borrow_lifetime() == *other
    }
}

impl Hash for LifetimeBorrow<'_> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.deref(), hasher)
    }
}

impl Deref for LifetimeBorrow<'_> {
    type Target = LifetimeData;
    fn deref(&self) -> &LifetimeData {
        if let Some(ptr) = &self.0 {
            &ptr
        } else {
            &STATIC_LIFETIME
        }
    }
}

impl Lifetime {
    /// The static `rain` lifetime
    pub const STATIC: Lifetime = Lifetime(None);
    /// Create a new `Lifetime` from `LifetimeData`
    pub fn new(data: LifetimeData) -> Lifetime {
        if data == STATIC_LIFETIME {
            return Lifetime(None);
        }
        Lifetime(Some(LIFETIME_CACHE.cache(data)))
    }
    /// Gets the lifetime for the nth parameter of a `Region`. Returns a regular lifetime `Region` on OOB
    #[inline]
    pub fn param(region: Region, ix: usize) -> Lifetime {
        Lifetime::new(LifetimeData::param(region, ix))
    }
    /// Deduplicate an `Arc<LifetimeData>` into a `Lifetime`
    pub fn dedup(arc: Arc<LifetimeData>) -> Lifetime {
        Lifetime(Some(LIFETIME_CACHE.cache(arc)))
    }
    /// Borrow this lifetime
    #[inline]
    pub fn borrow_lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow(self.0.as_ref().map(|v| v.borrow_arc()))
    }
    /// Check whether this lifetime is the static (null) lifetime
    #[inline]
    pub fn is_static(&self) -> bool {
        self.0.is_none()
    }
    /// Check whether this lifetime is idempotent, i.e. is equal to it's self intersection
    #[inline]
    pub fn idempotent(&self) -> bool {
        self.deref().idempotent()
    }
    /// Find the separating conjunction of this lifetime with itself.
    #[inline]
    pub fn star_self(&self) -> Result<(), Error> {
        self.deref().star_self()
    }
    /// Find the separating conjunction of this lifetime with another.
    #[inline]
    pub fn star(&self, other: &Lifetime) -> Result<Lifetime, Error> {
        if self == other {
            return self.star_self().map(|_| self.clone());
        }
        if self.is_static() {
            return Ok(other.clone());
        }
        if other.is_static() {
            return Ok(self.clone());
        }
        self.deref().star(other.deref()).map(Lifetime::new)
    }
    /// Find the conjunction of this lifetime with another
    #[inline]
    pub fn join(&self, other: &Lifetime) -> Result<Lifetime, Error> {
        if self == other || other.is_static() {
            return Ok(self.clone());
        }
        if self.is_static() {
            return Ok(other.clone());
        }
        self.deref().conj(other.deref()).map(Lifetime::new)
    }
    /// Find the conjunction of a set of lifetimes and this lifetime. Return an error if the lifetimes are incompatible.
    #[inline]
    pub fn conj<'a, I>(&'a self, lifetimes: I) -> Result<Lifetime, Error>
    where
        I: Iterator<Item = LifetimeBorrow<'a>>,
    {
        let mut base = self.clone();
        for lifetime in lifetimes {
            if lifetime != base {
                base = base.join(lifetime.as_lifetime())?;
            }
        }
        Ok(base)
    }
    /// Find the separating conjunction of a set of lifetimes and this lifetime. Return an error if the lifetimes are incompatible.
    #[inline]
    pub fn sep_conj<'a, I>(&'a self, lifetimes: I) -> Result<Lifetime, Error>
    where
        I: Iterator<Item = LifetimeBorrow<'a>>,
    {
        let mut base = self.clone();
        let mut base_idempotent = false;
        for lifetime in lifetimes {
            if lifetime != base {
                base = base.star(lifetime.as_lifetime())?;
                base_idempotent = false;
            } else if !base_idempotent {
                base.star_self()?;
                base_idempotent = true;
            }
        }
        Ok(base)
    }
    /// Escape a lifetime up to a given depth
    #[inline]
    pub fn escape_upto(&self, depth: usize) -> Lifetime {
        self.color_map(|_| &[], depth).expect("Null mapping cannot fail")
    }
    /// Escape a lifetime up to the current depth - 1
    #[inline]
    pub fn escape(&self) -> Lifetime {
        self.escape_upto(self.depth().saturating_sub(1))
    }
    /// Set a lifetime to be within a given region
    #[inline]
    pub fn in_region(&self, region: Option<Region>) -> Result<Lifetime, Error> {
        if let Some(data) = &self.0 {
            if data.region == region {
                // Avoid the hash table...
                return Ok(self.clone());
            }
            data.in_region(region).map(Lifetime::from)
        } else {
            Ok(region.into())
        }
    }
    /// Get a lifetime which owns only a given color
    #[inline]
    pub fn owns(color: Color) -> Lifetime {
        LifetimeData::owns(color).into()
    }
    /// Map the colors in a lifetime, up to a given depth
    #[inline]
    pub fn color_map<'a, F>(&self, color_map: F, depth: usize) -> Result<Lifetime, Error> where F: Fn(&Color) -> &'a [Color] {
        // Skip shallow lifetimes
        if self.depth() < depth {
            return Ok(self.clone());
        }
        // Map this lifetime's underlying data, and transform it to a Lifetime
        self.deref().color_map(color_map, depth).map(Lifetime::new)
    }
}

impl BitAnd for Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn bitand(self, other: Lifetime) -> Result<Lifetime, Error> {
        if self == other || other.is_static() {
            return Ok(self);
        }
        if self.is_static() {
            return Ok(other);
        }
        self.deref().conj(other.deref()).map(Lifetime::new)
    }
}

impl BitAnd<&'_ Lifetime> for Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn bitand(self, other: &Lifetime) -> Result<Lifetime, Error> {
        if self == *other || other.is_static() {
            return Ok(self);
        }
        if self.is_static() {
            return Ok(other.clone());
        }
        self.deref().conj(other.deref()).map(Lifetime::new)
    }
}

impl BitAnd<LifetimeBorrow<'_>> for Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn bitand(self, other: LifetimeBorrow) -> Result<Lifetime, Error> {
        self.bitand(other.as_lifetime())
    }
}

impl BitAnd<Lifetime> for &'_ Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn bitand(self, other: Lifetime) -> Result<Lifetime, Error> {
        other.bitand(self)
    }
}

impl BitAnd<&'_ Lifetime> for &'_ Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn bitand(self, other: &Lifetime) -> Result<Lifetime, Error> {
        other.join(self)
    }
}

impl BitAnd<LifetimeBorrow<'_>> for &'_ Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn bitand(self, other: LifetimeBorrow) -> Result<Lifetime, Error> {
        self.bitand(other.as_lifetime())
    }
}

impl BitAnd<Lifetime> for LifetimeBorrow<'_> {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn bitand(self, other: Lifetime) -> Result<Lifetime, Error> {
        self.as_lifetime().bitand(other)
    }
}

impl BitAnd<&'_ Lifetime> for LifetimeBorrow<'_> {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn bitand(self, other: &Lifetime) -> Result<Lifetime, Error> {
        self.as_lifetime().bitand(other)
    }
}

impl BitAnd<LifetimeBorrow<'_>> for LifetimeBorrow<'_> {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn bitand(self, other: LifetimeBorrow) -> Result<Lifetime, Error> {
        self.as_lifetime().bitand(other.as_lifetime())
    }
}

impl Mul for Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: Lifetime) -> Result<Lifetime, Error> {
        if self == other || other.is_static() {
            return Ok(self);
        }
        if self.is_static() {
            return Ok(other);
        }
        self.deref().conj(other.deref()).map(Lifetime::new)
    }
}

impl Mul<&'_ Lifetime> for Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: &Lifetime) -> Result<Lifetime, Error> {
        if self == *other || other.is_static() {
            return Ok(self);
        }
        if self.is_static() {
            return Ok(other.clone());
        }
        self.deref().conj(other.deref()).map(Lifetime::new)
    }
}

impl Mul<LifetimeBorrow<'_>> for Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: LifetimeBorrow) -> Result<Lifetime, Error> {
        self.mul(other.as_lifetime())
    }
}

impl Mul<Lifetime> for &'_ Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: Lifetime) -> Result<Lifetime, Error> {
        other.mul(self)
    }
}

impl Mul<&'_ Lifetime> for &'_ Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: &Lifetime) -> Result<Lifetime, Error> {
        other.join(self)
    }
}

impl Mul<LifetimeBorrow<'_>> for &'_ Lifetime {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: LifetimeBorrow) -> Result<Lifetime, Error> {
        self.mul(other.as_lifetime())
    }
}

impl Mul<Lifetime> for LifetimeBorrow<'_> {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: Lifetime) -> Result<Lifetime, Error> {
        self.as_lifetime().mul(other)
    }
}

impl Mul<&'_ Lifetime> for LifetimeBorrow<'_> {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: &Lifetime) -> Result<Lifetime, Error> {
        self.as_lifetime().mul(other)
    }
}

impl Mul<LifetimeBorrow<'_>> for LifetimeBorrow<'_> {
    type Output = Result<Lifetime, Error>;
    #[inline]
    fn mul(self, other: LifetimeBorrow) -> Result<Lifetime, Error> {
        self.as_lifetime().mul(other.as_lifetime())
    }
}

impl Regional for Lifetime {
    #[inline]
    fn region(&self) -> Option<RegionBorrow> {
        self.deref().region()
    }
}

impl From<LifetimeData> for Lifetime {
    #[inline]
    fn from(data: LifetimeData) -> Lifetime {
        Lifetime::new(data)
    }
}

impl From<Option<Region>> for Lifetime {
    #[inline]
    fn from(region: Option<Region>) -> Lifetime {
        region.map(Lifetime::from).unwrap_or(Lifetime(None))
    }
}

impl From<Region> for Lifetime {
    #[inline]
    fn from(region: Region) -> Lifetime {
        Lifetime::new(LifetimeData::from(region))
    }
}

impl PartialOrd for Lifetime {
    /**
    We define a lifetime to be a sublifetime of another lifetime if every value in one lifetime lies in the other,
    This naturally induces a partial ordering on the set of lifetimes.
    */
    fn partial_cmp(&self, other: &Lifetime) -> Option<Ordering> {
        self.deref().partial_cmp(other.deref())
    }
}

impl PartialOrd<LifetimeBorrow<'_>> for Lifetime {
    /**
    We define a lifetime to be a sublifetime of another lifetime if every value in one lifetime lies in the other,
    This naturally induces a partial ordering on the set of lifetimes.
    */
    fn partial_cmp(&self, other: &LifetimeBorrow<'_>) -> Option<Ordering> {
        self.deref().partial_cmp(other.deref())
    }
}

impl PartialOrd for LifetimeBorrow<'_> {
    /**
    We define a lifetime to be a sublifetime of another lifetime if every value in one lifetime lies in the other,
    This naturally induces a partial ordering on the set of lifetimes
    */
    fn partial_cmp(&self, other: &LifetimeBorrow<'_>) -> Option<Ordering> {
        self.deref().partial_cmp(other.deref())
    }
}

impl PartialOrd<Lifetime> for LifetimeBorrow<'_> {
    /**
    We define a lifetime to be a sublifetime of another lifetime if every value in one lifetime lies in the other,
    This naturally induces a partial ordering on the set of lifetimes.
    */
    fn partial_cmp(&self, other: &Lifetime) -> Option<Ordering> {
        self.deref().partial_cmp(other.deref())
    }
}

impl<'a> LifetimeBorrow<'a> {
    /// Clone this lifetime
    #[inline]
    pub fn clone_lifetime(&self) -> Lifetime {
        Lifetime(self.0.map(|v| v.clone_arc()))
    }
    /// Get this lifetime borrow as a lifetime
    #[inline]
    pub fn as_lifetime(&self) -> &Lifetime {
        unsafe { &*(self as *const _ as *const Lifetime) }
    }
    /// Get the region of this lifetime
    #[inline]
    pub fn get_region(&self) -> Option<RegionBorrow<'a>> {
        self.0.map(|r| r.get().region()).flatten()
    }
    /// Check whether this lifetime is the static (null) lifetime
    #[inline]
    pub fn is_static(&self) -> bool {
        self.0.is_none()
    }
}

impl Regional for LifetimeBorrow<'_> {
    #[inline]
    fn region(&self) -> Option<RegionBorrow> {
        self.deref().region()
    }
}

/// A trait implemented by values which have a lifetime
pub trait Live {
    /// Get the lifetime of this value
    fn lifetime(&self) -> LifetimeBorrow;
}

/// Implement `Regional` using `Live`'s `lifetime` function
#[macro_export]
macro_rules! lifetime_region {
    ($t:ty) => {
        impl $crate::region::Regional for $t {
            #[inline]
            fn region(&self) -> Option<$crate::region::RegionBorrow> {
                #[allow(unused_imports)]
                use $crate::lifetime::Live;
                self.lifetime().get_region()
            }
        }
    };
}

/// Implemented `Regional` and `Live` to return trivial values
#[macro_export]
macro_rules! trivial_lifetime {
    ($t:ty) => {
        impl $crate::region::Regional for $t {
            #[inline]
            fn region(&self) -> Option<$crate::region::RegionBorrow> {
                None
            }
            #[inline]
            fn cloned_region(&self) -> Option<$crate::region::Region> {
                None
            }
            #[inline]
            fn depth(&self) -> usize {
                0
            }
        }
        impl $crate::lifetime::Live for $t {
            #[inline]
            fn lifetime(&self) -> $crate::lifetime::LifetimeBorrow {
                $crate::lifetime::LifetimeBorrow::default()
            }
        }
    };
}
