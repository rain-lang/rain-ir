/*!
Sum types and injections
*/
use crate::function::pi::Pi;
use crate::lifetime::Lifetime;
use crate::value::{arr::ValArr, UniverseId, VarId};

/// A sum type
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Sum {
    /// The variants of this sum type
    variants: ValArr,
    /// The lifetime of this sum type
    lifetime: Lifetime,
    /// The type of this sum type
    ty: UniverseId,
}

/// An injection into a sum type
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Injection {
    /*
    /// The sum type being injected into
    target: VarId<Sum>,
    */
    /// The index of this injection
    ix: usize,
    /// The type of this injection
    ty: VarId<Pi>,
    /// The lifetime of this injection
    lifetime: Lifetime,
}
