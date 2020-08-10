/*!
Sum types and injections
*/
use crate::function::pi::Pi;
use crate::lifetime::Lifetime;
use crate::value::{
    arr::{ValArr, ValSet},
    UniverseId, VarId,
};

/// A sum type
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Sum {
    /// The variants of this sum type
    variants: ValArr,
    /// The lifetime of this sum type
    lifetime: Lifetime,
    /// The type of this sum type
    ty: UniverseId, //TODO: kind
}

/// A union type
/// 
/// TODO: think about this...
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Union {
    /// The members of this union
    members: ValSet,
    /// The lifetime of this union
    lifetime: Lifetime,
    /// The type of this union
    ty: UniverseId, //TODO: kind
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

//TODO: represent applications of injections as Sexprs or evaluate them somehow?
