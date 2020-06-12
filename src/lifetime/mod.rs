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
use std::cmp::Ordering;

use crate::region::{Region, RegionBorrow, Regional};

/// A `rain` lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct Lifetime(Region);

impl Lifetime {
    /// Borrow this lifetime
    #[inline]
    pub fn borrow_lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow(self.0.borrow_region())
    }
    /// Check whether this lifetime is the static (null) lifetime
    #[inline]
    pub fn is_static(&self) -> bool {
        self.0.is_null()
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
}

impl Regional for Lifetime {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.0.borrow_region()
    }
}

impl From<Region> for Lifetime {
    #[inline]
    fn from(region: Region) -> Lifetime {
        Lifetime(region)
    }
}

impl PartialOrd for Lifetime {
    /**
    We define a lifetime to be a sublifetime of another lifetime if every value in one lifetime lies in the other,
    This naturally induces a partial ordering on the set of lifetimes.
    */
    fn partial_cmp(&self, other: &Lifetime) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

/// A borrow of a `rain` lifetime
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct LifetimeBorrow<'a>(RegionBorrow<'a>);

impl PartialOrd for LifetimeBorrow<'_> {
    /**
    We define a lifetime to be a sublifetime of another lifetime if every value in one lifetime lies in the other,
    This naturally induces a partial ordering on the set of lifetimes
    */
    fn partial_cmp(&self, other: &LifetimeBorrow<'_>) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<'a> LifetimeBorrow<'a> {
    /// Clone this lifetime
    #[inline]
    pub fn clone_lifetime(&self) -> Lifetime {
        Lifetime(self.0.clone_region())
    }
    /// Get the region of this lifetime
    #[inline]
    pub fn get_region(&self) -> RegionBorrow<'a> {
        self.0
    }
    /// Check whether this lifetime is the static (null) lifetime
    #[inline]
    pub fn is_static(&self) -> bool {
        self.0.is_null()
    }
}

impl Regional for LifetimeBorrow<'_> {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.0
    }
}

impl<'a> From<RegionBorrow<'a>> for LifetimeBorrow<'a> {
    #[inline]
    fn from(borrow: RegionBorrow) -> LifetimeBorrow {
        LifetimeBorrow(borrow)
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
