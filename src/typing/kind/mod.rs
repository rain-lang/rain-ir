/*!
Meta-types and layouts
*/
use super::*;
use crate::value::{KindId, ReprId};

pub mod layout;
pub mod universe;

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
}

/// A trait implemented by `rain` values which can all be represented within a given memory layout
pub trait Repr: Kind {
    /// Convert this representation into a `ReprId`
    #[inline]
    fn into_repr(self) -> ReprId {
        self.into_val().coerce()
    }
}
