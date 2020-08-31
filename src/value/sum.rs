/*!
Sum types and injections
*/
use crate::function::pi::Pi;
use crate::value::{
    arr::{ValArr, ValSet},
    KindId, VarId,
};

/// A sum type
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Sum {
    /// The variants of this sum type
    variants: ValArr,
    /// The type of this sum type
    ty: KindId,
}

/// A union type
///
/// TODO: think about this...
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Union {
    /// The members of this union
    members: ValSet,
    /// The type of this union
    ty: KindId,
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
}

//TODO: represent applications of injections as Sexprs or evaluate them somehow?
