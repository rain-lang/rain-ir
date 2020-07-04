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

The `rain` intermediate representation consists of a DAG with values (represented by the `ValId` struct) as nodes and the dependencies
between values as edges. Every value in this graph is assigned a lifetime, and hence values which break the lifetime assignment rules
are impossible to construct: constructing them will return an error, since no valid lifetime can be assigned to them. Hence, rain's lifetime
rules can be described naturally by two questions:
- Which lifetimes exist?
- Which values can have which lifetimes?
Note the implication of the latter question: values cannot necessarily be assigned a unique lifetime, though they may sometimes have a *minimal*
lifetime (we will get to the partial order of lifetimes soon). That said, lifetime inference is part of `rain`-IR, and the even at the API/IR
level explicit, external lifetimes do not often need to be used (and often *cannot* be used, since certain values will generate thier own lifetimes).

## The Static Lifetime

The most basic lifetime is the static, null, or constant lifetime, which corresponds to a `NULL` lifetime pointer. "Constant lifetime" is probably the best
name, since this guarantees properties closer to Rust's `const` values than Rust values with `'static`. Values with the constant lifetime can be
freely copied and hence cannot have a destructor, and furthermore cannot depend on any non-constant values (e.g. function parameters, etc.).
Lifetimes are partially ordered, and the static lifetime is at the root of this partial order, i.e., is a minimal element: it is included in every lifetime, 
whereas no other lifetimes are included in the static lifetime. The partial order of lifetimes has no maximal element.

## Regions

Similar to the concept of scoping local variables, the `rain` graph, as an RVSDG, is divided into regions (represented by the `Region` struct). These regions
are partially ordered into a tree, with a null region at the root. Values in a region can use values in ancestor regions, but not vice versa. All values have
a single, unique, minimal region they can be placed into. All lifetimes, as well, can be placed into a region, and the lifetime assigned to a region must be
compatible with that value's minimal region: i.e., the value's minimal region *must* be an ancestor of the value's assigned lifetime's region.

In general, with some important exceptions, a value's region is the intersection of the regions of it's dependencies, where the intersection of a set of
regions is designed to be the largest region contained in every region in the set, if such a region exists. The most important exception to keep in mind
is values of the `ValueEnum::Parameter` variant, which serve as parameters into a region: these are assigned as region the region they are parameters into. 
If a value has dependencies that are incomparable (i.e. not ordered by inclusion), then it's lifetime is invalid, and hence it cannot be a valid `rain` value.

Note that functional variants like `ValueEnum::Lambda`, `ValueEnum::Gamma` and `ValueEnum::Pi` are *not* exceptions to the rule above: their lifetime is also
the intersection of the lifetimes of their dependencies. The caveat here is that their results do not count as dependencies, but rather the elements of
their `deps` vector do: in general, dependencies satisfy the property listed above that a node can *never* have a dependency in a region that is not it's
region or an ancestor.

A lifetime bound to a given region with no additional restrictions (e.g. linearity) is called a *region lifetime* or *frame lifetime* (in reference to stack frames).
`Copy` types like booleans which depend on a parameter to a region (including parameters with a `Copy` type) have a region lifetime. Values with non-`Copy`
types cannot have a region lifetime: see the section on linear lifetimes below.

### Regions as Quotients

It is possible to view regions as quotients of the lifetime graph: a value's *assigned* region is just the region of it's lifetime, and if we removed all
linear typing from `rain`, then a value's lifetime is just it's region (since all other lifetime information would be irrelevant). Note that according to
this definition, a values assigned region is distinct from it's minimal region: a value can be assigned a stronger lifetime that would place it into a
region nested within it's minimal region. From this point of view, then, the region system is just the lifetime system modulo linear types, and in fact
it could be possible to implement the region system purely in terms of lifetimes in this manner. However, for simplicity, the region system is a simpler
backbone which the lifetime system is built upon.

## Substructural Types

One of the core features of `rain` is a [substructural type system](https://mitpress-request.mit.edu/sites/default/files/titles/content/9780262162289_sch_0001.pdf) 
which, as we  will see below, allows us to naturally represent stateful operations common in imperative programs (such as calls to external libraries, IO, 
mutable state, and manual memory management) in the purely functional context of `rain`, allowing easy translation of the `rain` IR to and from imperative frontends 
(like, someday perhaps, C and Rust) and backends (like LLVM and WASM).

A *linear type* is a type such that it's values must be used exactly once. `rain` also supports other types corresponding to different forms of substructural 
logic, including
- *Affine types*, which can be used at most once but can remain unused
- *Relevant types*, which must be used at least once but can be used multiple times

A type without any such usage restriction is called an *unrestricted* type. Note that linear types are both affine and relevant, but not all affine/relevant types
are linear. For example, in Rust terms, `Vec<T>` is affine (it can only be used once), but not relevant (not even necessarily ignoring `Drop`, if we take into account
`mem::forget` or even leak amplification) and therefore is *not* linear.

*Ordered types*, which are types that must be used exactly once in the order of their introduction, are supported in a very limited sense, namely,
pi-types with a linear type parameter must use this parameter in their return type exactly once, and hence, a nested list of such pi types acts like
somewhat like a fragment of ordered logic (we note that the linearity of the type means it will be consumed by the pi type and hence unable to be used 
by the lambda function it types). That said, because in general `rain` IR is a graph and hence has no order of variable introduction, there cannot be 
any other support for ordered typing.

A value with a linear type is assigned a *linear lifetime*. Linear typing is then enforced in two ways
- Via incompatibility between lifetimes: these are called the "logical lifetime rules"
- Via "escape inconsistency", which is where a value can be constructed but cannot escape from it's defining region via an escape constructor like
`#lambda` or `#pi`.

Separating enforcement in this manner has both theoretical and practical benefits: theoretically, for example, it is not wrong to have *a* value which
does not use a relevant type (which must be used at least once), but it is definitely wrong to have a value be the result of a lambda function where that
lambda function either has a relevant type as a paremeter or as an unused result of one of the operations in it's definition. Hence, relevant types, and
the relevancy restriction of a linear type, can only be enforced by escape inconsistency and not by lifetime incompatibility.

### Logical Lifetime Rules

Substructural lifetimes can come from the value or region level:
- A *constructor* for a substructural lifetime can create a value with a lifetime equal to the intersection of the dependencies of that value and 
a fresh substructural lifetime. For example, a constructor for allocated memory can consume a reference to an allocator to yield a fresh affinely-typed.
Any function can be made constructor like by enforcing that it does the same.
- A *parameter* of a substructural type can have a substructural lifetime enforced by it's associated `Region`.

In general:
- An affine lifetime is incompatible with itself, so a value cannot have two dependencies with the same affine lifetime
- If lifetimes `A` and `B` are incompatible, and `C` is a sublifetime of `B`, then `A` and `C` are incompatible

### Lifetime Inconsistency

Lifetime inconsistency, similarly, is also checked at the value and region level:
- At the *value* level, if an operation produces a product type with a relevant member, then that member must be used by the result of any function-variant
escaping the region of the operation
- At the *region* level, the result of any function-variant must use all relevant parameters of it's region at least once

Another layer of lifetime inconsistency checking performed is borrow checking, which is described below.

## Borrow Lifetimes

As in Rust, the borrow checker is a central part of `rain`'s functionality. In brief, it allows lifetimes to "borrow" from values, creating artificial
lifetime incompatibilites which mimic Rust's imperative borrow checking.

TODO: rest

## Planned: Cellular Lifetimes

While borrow lifetimes describe immutable borrows and, as shown in the examples below, mutable borrows can be represented with linear types, interior
mutability, `Cell` and atomics still remain to be covered. We are currently pursuing a framework for these based off concurrent separation logic which
is fully general, however, we plan to design a specialized system of "cellular lifetimes" for this case which can simplify optimizing certain cases.

TODO

# Basic Lifetime Examples

## Non-`Copy` objects

Consider as an example the following simple Rust program:
```ignore
let x: String = "Hello".into();
let y = x + "!";
```
Giving this `rain`-like lifetimes...

We know that adding another usage of `x` to this program, like
```
let z = x + "?";
```
would yield an invalid program.

TODO

## Immutable Borrows

Consider as an example the following simple Rust program:
```ignore
let x: String = "Hello".into();
println!("{}", x);
let y = x + "!";
```

TODO

## Mutable Borrows

TODO

## "Linear types can change the world!"

TODO

## Lifetimes as Quotients

Just as regions can be viewed as quotients of the lifetime system, we can actually view lifetimes themselves as quotients of the `rain` graph.
This, in particular, yields an algorithm for doing a partial lifetime check of the `rain` graph on just `rain` values, though some lifetime
annotation for parameters to functions/regions is still needed.

TODO

# Allocators and Functional Drop

## Allocator support

TODO

## Nondeterministic Drops and the stack

TODO

## Example: garbage collection

TODO

# Interior Mutability

TODO

# Unsafe Code

## Assumptions

TODO

## Heaps and Raw Pointers

TODO

## Heap Combinators

TODO

# Concurrency and Atomics

## Atomic Cell Lifetimes

TODO

## Mutexes

TODO

## Concurrent Heap Combinators

TODO

## The `Time` object

TODO

# Compiling Lifetimes

## Virtual Dependencies

TODO

## Alias Analysis

TODO

# Rust Lifetimes vs. `rain` Lifetimes

## HIR lifetimes and Regions

TODO

## Aliasing

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
