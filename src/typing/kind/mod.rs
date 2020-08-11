/*!
Meta-types and layouts
*/
use super::*;

pub mod universe;

/// A trait implemented by `rain` values which are a kind, i.e. a type of types
pub trait Kind: Type {}

/// A trait implemented by `rain` values which can all be represented within a given memory layout
pub trait Repr: Kind {}
