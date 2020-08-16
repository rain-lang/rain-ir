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

/// A predicate indicating a `rain` value is a kind
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct IsKind;

impl TypePredicate for IsKind {}

/// A predicate which indicates that a `rain` value is a kind
pub trait KindPredicate: TypePredicate {}

impl KindPredicate for IsKind {}

impl<K> KindPredicate for Is<K> where K: Kind {}

/// A predicate indicating a `rain` value is a representation
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct IsRepr;

impl TypePredicate for IsRepr {}

impl KindPredicate for IsRepr {}

/// A predicate which indicates that a `rain` value is a type
pub trait ReprPredicate: KindPredicate {}

impl ReprPredicate for IsRepr {}

impl<R: Repr> ReprPredicate for Is<R> {}

/// A predicate indicating a `rain` value is a representation
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct IsUniverse;

impl TypePredicate for IsUniverse {}

impl KindPredicate for IsUniverse {}

/// A predicate which indicates that a `rain` value is a universe
pub trait UniversePredicate: KindPredicate {}

impl UniversePredicate for IsUniverse {}

impl<U: Universe> UniversePredicate for Is<U> {}
