/*!
`rain` value lifetimes

`rain`'s lifetime system centers around the `Lifetime` object, which semantically represents a distinct `rain` lifetime.
`Lifetime` objects are automatically managed and garbage collected by a global lifetime cache. This module contains the
definitions for the `Lifetime` object, the lifetime cache, a variety of auxiliary objects (e.g. `LifetimeBorrow` to avoid
pointer-chasing) and implementations of a variety of algorithms used in the lifetime system.

*/
use crate::region::{Region, RegionBorrow, Regional};
use crate::value::{Error, ValId};
use dashcache::{DashCache, GlobalCache};
use elysees::{Arc, ArcBorrow};
use lazy_static::lazy_static;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::ops::{Add, Mul};

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

impl Deref for Lifetime {
    type Target = LifetimeData;
    fn deref(&self) -> &LifetimeData {
        if let Some(ptr) = &self.0 {
            &ptr
        } else {
            &STATIC_LIFETIME_DATA
        }
    }
}

impl Lifetime {
    /// The static `rain` lifetime
    pub const STATIC: Lifetime = Lifetime(None);
    /// Create a new lifetime from given data
    #[inline]
    pub fn new(data: LifetimeData) -> Lifetime {
        if data.is_static() {
            Self::STATIC
        } else {
            Lifetime(Some(LIFETIME_CACHE.cache(data)))
        }
    }
    /// Get the lifetime associated with a single parameter of a given region
    #[inline]
    pub fn param(region: Region, ix: usize) -> Result<Lifetime, Error> {
        LifetimeData::param(region, ix).map(Lifetime::new)
    }
    /// Get this lifetime in a given region
    #[inline]
    pub fn in_region(&self, region: Option<Region>) -> Result<Lifetime, Error> {
        if let Some(data) = self.data() {
            Ok(Lifetime::new(data.in_region(region)?))
        } else {
            Ok(Lifetime::from(region))
        }
    }
    /// Get a lifetime which owns a single color
    #[inline]
    pub fn owns(color: Color) -> Lifetime {
        LifetimeData::owns(color).into()
    }
    /// Borrow a lifetime
    #[inline]
    pub fn borrow_lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow(self.0.as_ref().map(Arc::borrow_arc))
    }
    /// Get the data backing this lifetime, if any
    #[inline]
    pub fn data(&self) -> Option<&LifetimeData> {
        self.0.as_deref()
    }
    /// Get a pointer to this lifetime's data, if any, or null otherwise
    #[inline]
    pub fn data_ptr(&self) -> *const LifetimeData {
        if let Some(data) = self.data() {
            data
        } else {
            std::ptr::null()
        }
    }
    /// Get the data backing this lifetime
    #[inline]
    pub fn data_or_static(&self) -> &LifetimeData {
        self.data().unwrap_or(&STATIC_LIFETIME_DATA)
    }
    /// Take the separating conjunction of two lifetimes
    #[inline]
    pub fn sep_conj(&self, other: &Lifetime) -> Result<Lifetime, Error> {
        match (self.0.as_ref(), other.0.as_ref()) {
            (None, None) => Ok(Lifetime::STATIC),
            (Some(_), None) => Ok(self.clone()),
            (None, Some(_)) => Ok(other.clone()),
            (Some(l), Some(r)) => (l.sep_conj(r)).map(Lifetime::new),
        }
    }
    /// Take the disjunction of two lifetimes
    #[inline]
    pub fn disj(&self, other: &Lifetime) -> Result<Lifetime, Error> {
        match (self.0.as_ref(), other.0.as_ref()) {
            (None, None) => Ok(Lifetime::STATIC),
            (Some(_), None) => Ok(self.affine_component()),
            (None, Some(_)) => Ok(other.affine_component()),
            (Some(l), Some(r)) => (l.disj(r)).map(Lifetime::new),
        }
    }
    /// Get the affine component of this lifetime
    #[inline]
    pub fn affine_component(&self) -> Lifetime {
        if let Some(data) = self.data() {
            Lifetime::new(data.affine_component())
        } else {
            Lifetime::STATIC
        }
    }
    /// Get the relevant component of this lifetime
    #[inline]
    pub fn relevant_component(&self) -> Lifetime {
        if let Some(data) = self.data() {
            Lifetime::new(data.relevant_component())
        } else {
            Lifetime::STATIC
        }
    }
    /// Accumulate a lifetime under separating conjunction
    ///
    /// Leave this lifetime in an undetermined but valid state on failure
    #[inline]
    pub fn sep_conj_acc<'a, L>(&mut self, lifetimes: L) -> Result<(), Error>
    where
        L: Iterator<Item = LifetimeBorrow<'a>>,
    {
        for lifetime in lifetimes {
            *self = self.sep_conj(&*lifetime)?;
        }
        Ok(())
    }
    /// Accumulate a lifetime under disjunction
    ///
    /// Leave this lifetime in an undetermined but valid state on failure
    #[inline]
    pub fn disj_acc<'a, L>(&mut self, lifetimes: L) -> Result<(), Error>
    where
        L: Iterator<Item = LifetimeBorrow<'a>>,
    {
        for lifetime in lifetimes {
            *self = self.disj(&*lifetime)?;
        }
        Ok(())
    }
    /// Take the separating conjunction of a set of lifetimes
    #[inline]
    pub fn sep_conjs<'a, L>(&self, lifetimes: L) -> Result<Lifetime, Error>
    where
        L: Iterator<Item = LifetimeBorrow<'a>>,
    {
        let mut result = self.clone();
        result.sep_conj_acc(lifetimes)?;
        Ok(result)
    }
    /// Take the disjunction of a set of lifetimes
    #[inline]
    pub fn disjs<'a, L>(&self, lifetimes: L) -> Result<Lifetime, Error>
    where
        L: Iterator<Item = LifetimeBorrow<'a>>,
    {
        let mut result = self.clone();
        result.disj_acc(lifetimes)?;
        Ok(result)
    }
    /// Attempt to color map a lifetime while truncating it's region to a given level
    #[inline]
    pub fn color_map<'a, F, P>(
        &self,
        color_map: F,
        parametric_map: P,
        depth: usize,
    ) -> Result<Lifetime, Error>
    where
        F: FnMut(&Color) -> Option<&'a Lifetime>,
        P: FnMut(&ValId) -> Result<ValId, Error>,
    {
        if let Some(data) = self.data() {
            let mut data = data.clone();
            data.color_map(color_map, parametric_map, depth)?;
            Ok(Lifetime::new(data))
        } else {
            Ok(Lifetime::STATIC)
        }
    }
}

impl Deref for LifetimeBorrow<'_> {
    type Target = Lifetime;
    fn deref(&self) -> &Lifetime {
        self.as_lifetime()
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
    /// Get the data backing this lifetime, if any
    #[inline]
    pub fn data(&self) -> Option<&'a LifetimeData> {
        self.0.map(|l| l.get())
    }
    /// Get a pointer to this lifetime's data, if any, or null otherwise
    #[inline]
    pub fn data_ptr(&self) -> *const LifetimeData {
        if let Some(data) = self.data() {
            data
        } else {
            std::ptr::null()
        }
    }
    /// Get the data backing this lifetime
    #[inline]
    pub fn data_or_static(&self) -> &'a LifetimeData {
        self.data().unwrap_or(&STATIC_LIFETIME_DATA)
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
