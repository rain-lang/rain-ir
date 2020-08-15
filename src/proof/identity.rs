/*!
Proofs of identity and equivalence.
*/
use crate::lifetime::Lifetime;
use crate::value::{arr::ValSet, TypeId, ValId};

/// A proof of identity for two values
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Id {
    /// The left value being compared
    left: ValId,
    /// The right value being compared
    right: ValId,
    /// The type of this identity type
    ty: TypeId,
    /// The lifetime of this identity type
    lt: Lifetime,
}

/// A proof of identity for a set of values, where the size of the set is less than or equal to 2
/// 
/// Values of this type can only be constructed where the type of the values is of kind `#set`, implying identity is a mere proposition.
/// In this case `IdBinSet` is *always* a mere proposition.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct IdBinSet {
    /// The high value being compared
    high: ValId,
    /// The low value being compared, if any
    low: Option<ValId>,
    /// The type of this identity type
    ty: TypeId,
    /// The lifetime of this identity type
    lt: Lifetime,
}

/// A proof of identity for a set of values
///
/// Values of this type can only be constructed where the type of the values is of kind `#set`, implying identity is a mere proposition.
/// In this case `IdSet` is *always* a mere proposition.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct IdSet {
    /// The values being compared
    id_set: ValSet,
    /// The type of this identity type
    ty: TypeId,
    /// The lifetime of this identity type
    lt: Lifetime,
}
