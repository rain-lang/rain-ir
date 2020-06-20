/*!
`rain` value lifetimes

# Introduction

`rain` at it's core is an [RVSDG](https://arxiv.org/abs/1912.05036) with dependent linear types generalized with lifetimes.
In Rust, lifetimes can be modeled using the formal, imperative semantics of [Stacked Borrows](https://plv.mpi-sws.org/rustbelt/stacked-borrows/),
however, since `rain` is a purely functional language, statements can be executed in an arbitrary order (including in parallel) constrained only by their
dependencies. Hence, in `rain`, we instead model lifetimes a set of conditions on possible dependencies between values, and hence as a
generalization of linear types. That said, it is occasionally useful to think of `rain` lifetimes as
[stacked borrows](https://plv.mpi-sws.org/rustbelt/stacked-borrows/) applied not to a particular imperative program,
but to the equivalence class of all imperative programs executing the instructions of a `rain` program in an order
satisfying certain constraints. It is important to keep in mind, however, that there are subtle differences between these models,
such that something allowable in one may be forbidden in the other.

We can sum up the basic idea behind rain's lifetime system in a single sentence: "values cannot be referenced after they are used". Remember,
since `rain` is purely functional, there is no concept of mutable borrows (unlike in Rust): instead, there are updates which are marked so as
to be compiled as in-place operations. To model more complex systems, such as those involving interior mutability, we will later introduce
*heap types* based off [Concurrent Separation Logic](https://dl.acm.org/doi/10.1145/2984450.2984457), but for now, we will focus purely on
immutable borrows. One key difference between Rust's immutable borrows and `rain`'s immutable borrows is that the latter are not constrained
to have the same representation as a pointer: for example, if we were to borrow the `rain` equivalent of a `Vec`, both a bitwise copy of the
`Vec`'s contents and a pointer to the `Vec` would be considered valid borrows, and treated the same by the borrow checker (we note that they
would have the same lifetime, but different types). This, of course, can be achieved in Rust through appropriate use of `PhantomData` and
unsafe code, but I believe having it as a language feature is much cleaner. Types which must maintain a constant address across borrows,
and pinning, is dealt with by an "address dependency" system in which a given value type has an implicit dependency on its address, treated
almost like a field, but again, for our current, simple case, we will ignore this subtlety.

*/
use crate::region::{Region, RegionBorrow, Regional};
use crate::util::cache::Cache;
use lazy_static::lazy_static;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use triomphe::{Arc, ArcBorrow};

mod arr;
pub use arr::*;

lazy_static! {
    /// The global lifetime cache
    pub static ref LIFETIME_CACHE: Cache<LifetimeData> = Cache::new();
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
