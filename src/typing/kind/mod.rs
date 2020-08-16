/*!
Meta-types and layouts
*/
use super::*;
use crate::value::{KindId, ReprId, ValId, ValRef};

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

impl<K: KindPredicate> Kind for ValId<K> {
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
}
