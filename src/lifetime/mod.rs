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
use std::ops::{Add, BitAnd, Mul};

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
mod lifetime_impl;

lazy_static! {
    /// The global lifetime cache
    pub static ref LIFETIME_CACHE: DashCache<Arc<LifetimeData>> = DashCache::new();
}

/// A `rain` lifetime
#[derive(Debug, Clone, Eq, Default)]
#[repr(transparent)]
pub struct Lifetime(Option<Arc<LifetimeData>>);

/// A borrow of a `rain` lifetime
#[derive(Debug, Copy, Clone, Eq, Default)]
pub struct LifetimeBorrow<'a>(Option<ArcBorrow<'a, LifetimeData>>);

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
        self.color_map(|_| &[], depth)
            .expect("Null mapping cannot fail")
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
    pub fn color_map<'a, F>(&self, color_map: F, depth: usize) -> Result<Lifetime, Error>
    where
        F: Fn(&Color) -> &'a [Color],
    {
        // Skip shallow lifetimes
        if self.depth() < depth {
            return Ok(self.clone());
        }
        // Map this lifetime's underlying data, and transform it to a Lifetime
        self.deref().color_map(color_map, depth).map(Lifetime::new)
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
