/*!
Meta-types and layouts
*/
use super::*;

pub mod universe;

/// A trait implemented by `rain` values which are a kind, i.e. a type of types
pub trait Kind {}

/// A `ValueEnum` which is known to be a kind
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct KindValue(ValueEnum);

/// A trait implemented by `rain` values which can all be represented within a given memory layout
pub trait Repr: Kind {}

/// A `ValueEnum` which is known to be a representation
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct ReprValue(ValueEnum);