/*!
`rain` value lifetimes

# Module Overview

`rain`'s lifetime system centers around the `Lifetime` object, which semantically represents a distinct `rain` lifetime.
`Lifetime` objects are automatically managed and garbage collected by a global lifetime cache. This module contains the
definitions for the `Lifetime` object, the lifetime cache, a variety of auxiliary objects (e.g. `LifetimeBorrow` to avoid 
pointer-chasing) and implementations of a variety of algorithms used in the lifetime system.

# Introduction

`rain` is fundamentally an [RVSDG](https://arxiv.org/abs/1912.05036) extended with a concept of lifetimes, inspired but distinct
from the lifetimes in Rust. Unlike Rust (or rather, Rust's [MIR](https://rustc-dev-guide.rust-lang.org/mir/index.html), which is a 
more appropriate comparison point), `rain` is a purely functional language, and hence it is unwieldy to define lifetimes in terms 
of the execution order of statements as is done, e.g., in the [Stacked Borrows model](https://plv.mpi-sws.org/rustbelt/stacked-borrows/paper.pdf). 
Instead, lifetimes are defined purely in terms of the dependency graph. This has the additional effect of making lifetimes "concurrency agnostic:" 
as the dependency graph in general makes no assumption as to whether it is evaluated concurrently or sequentially (outside of nodes merging disjoint
sections of it according to [Concurrent Separation Logic](https://read.seas.harvard.edu/~kohler/class/cs260r-17/brookes16concurrent.pdf))
this definition naturally encompasses both concurrent and sequential programs by taking advantage of the properties of an RVSDG.

The current lifetime system in `rain` is not yet completely formalized, as the semantics of the language are still being developed.
What is documented here is an informal summary of the current state of the lifetime model we plan to implement in the prototype
`rain-ir` interpreter: this model is highly incomplete, and many important features remain unfinished. Currently, we only implement the
simplest kind of lifetime, namely a region or frame lifetime. This form of lifetime is akin to Rust's original syntactic lifetime model,
and indeed, with linear types, we hypothesize that it can simulate a good portion of the rest of the lifetime system on it's own, though
this would require compiling many, many inlined lambda functions. 

# Lifetimes

TODO

# Basic Lifetime Examples

TODO

# Interior Mutability

TODO

# Concurrency and Atomics

TODO

# Compiling Lifetimes

TODO

# Rust Lifetimes vs. `rain` Lifetimes

TODO

# Implementing Basic Lifetimes with Frames

TODO

*/
use crate::region::{Region, RegionBorrow, Regional};
use dashcache::{DashCache, GlobalCache};
use elysees::{Arc, ArcBorrow};
use lazy_static::lazy_static;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

mod arr;
pub use arr::*;

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

/// The data describing a `rain` lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd)]
pub enum LifetimeData {
    /// A region. TODO: this
    Region(Region),
}

/// The static `rain` lifetime, with a constant address
pub static STATIC_LIFETIME: LifetimeData = LifetimeData::Region(Region::NULL);

impl Lifetime {
    /// The static `rain` lifetime
    pub const STATIC: Lifetime = Lifetime(None);
    /// Create a new `Lifetime` from `LifetimeData`
    pub fn new(data: LifetimeData) -> Lifetime {
        Lifetime(Some(LIFETIME_CACHE.cache(data)))
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
    /// Find the intersection of a set of lifetimes and this lifetime. Return an error if the lifetimes are incompatible.
    #[inline]
    pub fn intersect<'a, I>(&'a self, lifetimes: I) -> Result<Lifetime, ()>
    where
        I: Iterator<Item = LifetimeBorrow<'a>>,
    {
        let mut base = self.borrow_lifetime();
        for lifetime in lifetimes {
            if let Some(ord) = base.partial_cmp(&lifetime) {
                if ord == Ordering::Less {
                    base = lifetime
                }
            } else {
                //TODO: lifetime intersections where possible...
                return Err(()); // Incompatible regions!
            }
        }
        Ok(base.clone_lifetime())
    }
    /// Escape a lifetime up to a given depth
    #[inline]
    pub fn escape_upto(&self, depth: usize) -> Lifetime {
        if self.depth() <= depth {
            return self.clone();
        }
        self.region().ancestor(depth).clone_region().into()
    }
    /// Escape a lifetime up to the current depth - 1
    #[inline]
    pub fn escape(&self) -> Lifetime {
        self.escape_upto(self.depth().saturating_sub(1))
    }
}

impl Regional for Lifetime {
    #[inline]
    fn region(&self) -> RegionBorrow {
        match self.deref() {
            LifetimeData::Region(r) => r.borrow_region(),
        }
    }
}

impl From<Region> for Lifetime {
    #[inline]
    fn from(region: Region) -> Lifetime {
        if region.is_null() {
            Lifetime(None)
        } else {
            Lifetime::new(LifetimeData::Region(region).into())
        }
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
    /// Get the region of this lifetime
    #[inline]
    pub fn get_region(&self) -> RegionBorrow<'a> {
        match self.0 {
            None => RegionBorrow::NULL,
            Some(r) => match r.get() {
                LifetimeData::Region(r) => r.borrow_region(),
            },
        }
    }
    /// Check whether this lifetime is the static (null) lifetime
    #[inline]
    pub fn is_static(&self) -> bool {
        self.0.is_none()
    }
}

impl Regional for LifetimeBorrow<'_> {
    #[inline]
    fn region(&self) -> RegionBorrow {
        match self.deref() {
            LifetimeData::Region(r) => r.borrow_region(),
        }
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
            fn region(&self) -> $crate::region::RegionBorrow {
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
            fn region(&self) -> $crate::region::RegionBorrow {
                $crate::region::RegionBorrow::default()
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
