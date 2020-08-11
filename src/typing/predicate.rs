/*!
Typing predicates
*/
use super::*;
use crate::value::predicate::Is;

/// A predicate indicating a `rain` value is a type
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct IsType;

/// A predicate which indicates that a `rain` value is a type
pub trait TypePredicate {}

impl TypePredicate for IsType {}

impl<T> TypePredicate for Is<T> where T: Type {}