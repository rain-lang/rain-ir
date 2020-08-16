/*!
Meta-types and layouts
*/
use super::*;
use crate::value::{KindId, ReprId, UniverseId, UniverseRef, ValId, ValRef};
use std::cmp::Ordering;

pub mod layout;
pub mod primitive;

/// A trait implemented by `rain` values which are a kind, i.e. a type of types
pub trait Kind: Type {
    /// Convert this kind into a `KindId`
    ///
    /// # Correctness
    /// The result of this method should always be pointer equivalent to `self.into_val()`
    #[inline]
    fn into_kind(self) -> KindId {
        self.into_val().coerce()
    }
    /// Get the kind of identity families over this kind
    fn id_kind(&self) -> KindId;
    /// Try to get this value's closure under the primitive type formers as a reference
    fn try_closure(&self) -> Option<UniverseRef> {
        None
    }
    /// Get the closure of this kind under the primitive type formers
    ///
    /// This is guaranteed to be a universe which has this kind as a subtype. If this kind is a universe,
    /// then this is guaranteed to just return this kind as a `UniverseId`
    fn closure(&self) -> UniverseId;
}

/// A trait implemented by `rain` values which can all be represented within a given memory layout
pub trait Repr: Kind {
    /// Convert this representation into a `ReprId`
    ///
    /// # Correctness
    /// The result of this method should always be pointer equivalent to `self.into_val()`
    #[inline]
    fn into_repr(self) -> ReprId {
        self.into_val().coerce()
    }
}

/// A trait implemented by `rain` values which are closed under the primitive type formers, namely Pi and Sigma
///
/// Universes also form a strict order, in that given two universes, one always strictly encloses the other.
/// This *may* be relaxed to a lattice property in later versions
pub trait Universe: Kind {
    /// Convert this representation into a `UniverseId`
    ///
    /// # Correctness
    /// The result of this method should always be pointer equivalent to `self.into_val()`
    #[inline]
    fn into_universe(self) -> UniverseId {
        self.into_val().coerce()
    }
    /// Compare two universes
    fn universe_cmp(&self, other: &UniverseId) -> Ordering;
}

impl<K: KindPredicate> Kind for ValId<K> {
    fn id_kind(&self) -> KindId {
        match self.as_enum() {
            ValueEnum::Prop(p) => p.id_kind(),
            ValueEnum::Fin(f) => f.id_kind(),
            ValueEnum::Set(s) => s.id_kind(),
            ValueEnum::Sexpr(s) => unimplemented!("Sexpr id kinds for {}", s),
            ValueEnum::Parameter(p) => unimplemented!("Parameter id kinds for {}", p),
            v => panic!("{} is not a kind!", v),
        }
    }
    fn try_closure(&self) -> Option<UniverseRef> {
        if self.is_universe() {
            return Some(self.borrow_var().coerce());
        }
        match self.as_enum() {
            ValueEnum::Prop(p) => p.try_closure(),
            ValueEnum::Fin(f) => f.try_closure(),
            ValueEnum::Set(s) => s.try_closure(),
            ValueEnum::Sexpr(s) => unimplemented!("Sexpr kind closure for {}", s),
            ValueEnum::Parameter(p) => unimplemented!("Parameter kind closure for {}", p),
            v => panic!("{} is not a kind!", v),
        }
    }
    fn closure(&self) -> UniverseId {
        match self.as_enum() {
            ValueEnum::Prop(p) => p.closure(),
            ValueEnum::Fin(f) => f.closure(),
            ValueEnum::Set(s) => s.closure(),
            ValueEnum::Sexpr(s) => unimplemented!("Sexpr kind closure for {}", s),
            ValueEnum::Parameter(p) => unimplemented!("Parameter kind closure for {}", p),
            v => panic!("{} is not a kind!", v),
        }
    }
}

impl<'a, K: KindPredicate> Kind for ValRef<'a, K> {
    fn id_kind(&self) -> KindId {
        match self.as_enum() {
            ValueEnum::Prop(p) => p.id_kind(),
            ValueEnum::Fin(f) => f.id_kind(),
            ValueEnum::Set(s) => s.id_kind(),
            ValueEnum::Sexpr(s) => unimplemented!("Sexpr kinds for {}", s),
            ValueEnum::Parameter(p) => unimplemented!("Parameter kinds for {}", p),
            v => panic!("{} is not a kind!", v),
        }
    }
    fn try_closure(&self) -> Option<UniverseRef> {
        if self.is_universe() {
            return Some(self.coerce());
        }
        match self.as_enum() {
            ValueEnum::Prop(p) => p.try_closure(),
            ValueEnum::Fin(f) => f.try_closure(),
            ValueEnum::Set(s) => s.try_closure(),
            ValueEnum::Sexpr(s) => unimplemented!("Sexpr kind closure for {}", s),
            ValueEnum::Parameter(p) => unimplemented!("Parameter kind closure for {}", p),
            v => panic!("{} is not a kind!", v),
        }
    }
    fn closure(&self) -> UniverseId {
        match self.as_enum() {
            ValueEnum::Prop(p) => p.closure(),
            ValueEnum::Fin(f) => f.closure(),
            ValueEnum::Set(s) => s.closure(),
            ValueEnum::Sexpr(s) => unimplemented!("Sexpr kind closure for {}", s),
            ValueEnum::Parameter(p) => unimplemented!("Parameter kind closure for {}", p),
            v => panic!("{} is not a kind!", v),
        }
    }
}

impl<U: UniversePredicate> Universe for ValId<U> {
    /// Compare two universes
    #[inline]
    fn universe_cmp(&self, other: &UniverseId) -> Ordering {
        match self.as_enum() {
            ValueEnum::Prop(p) => p.universe_cmp(other),
            ValueEnum::Fin(f) => f.universe_cmp(other),
            ValueEnum::Set(s) => s.universe_cmp(other),
            v => panic!("Value {} asserted to be a universe, but is not!", v),
        }
    }
}

impl<'a, U: UniversePredicate> Universe for ValRef<'a, U> {
    /// Compare two universes
    #[inline]
    fn universe_cmp(&self, other: &UniverseId) -> Ordering {
        match self.as_enum() {
            ValueEnum::Prop(p) => p.universe_cmp(other),
            ValueEnum::Fin(f) => f.universe_cmp(other),
            ValueEnum::Set(s) => s.universe_cmp(other),
            v => panic!("Value {} asserted to be a universe, but is not!", v),
        }
    }
}

impl PartialOrd for UniverseId {
    #[inline]
    fn partial_cmp(&self, other: &UniverseId) -> Option<Ordering> {
        Some(self.universe_cmp(other))
    }
}

impl Ord for UniverseId {
    #[inline]
    fn cmp(&self, other: &UniverseId) -> Ordering {
        self.universe_cmp(other)
    }
}

impl PartialOrd for UniverseRef<'_> {
    #[inline]
    fn partial_cmp(&self, other: &UniverseRef) -> Option<Ordering> {
        Some(self.universe_cmp(other.as_arc()))
    }
}

impl Ord for UniverseRef<'_> {
    #[inline]
    fn cmp(&self, other: &UniverseRef) -> Ordering {
        self.universe_cmp(other.as_arc())
    }
}

impl PartialOrd<UniverseId> for UniverseRef<'_> {
    #[inline]
    fn partial_cmp(&self, other: &UniverseId) -> Option<Ordering> {
        Some(self.universe_cmp(other))
    }
}

impl PartialOrd<UniverseRef<'_>> for UniverseId {
    #[inline]
    fn partial_cmp(&self, other: &UniverseRef) -> Option<Ordering> {
        Some(self.universe_cmp(other.as_arc()))
    }
}

