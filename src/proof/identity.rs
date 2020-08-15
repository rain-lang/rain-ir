/*!
Proofs of identity and equivalence.
*/
use crate::function::pi::Pi;
use crate::lifetime::Lifetime;
use crate::region::Regional;
use crate::typing::{universe::FINITE_TY, Typed};
use crate::value::{Error, /*arr::ValSet,*/ TypeId, ValId, VarId};
//use either::Either;

/// The identity type family for a given type
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct IdFamily {
    /// The type being parametrized
    param_ty: TypeId,
    /// The type of this family
    ty: VarId<Pi>,
    /// The lifetime of this family
    lt: Lifetime,
}

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

impl Id {
    /// Get the reflexivity type for a given value
    pub fn refl(value: ValId) -> Id {
        let lt = value.cloned_region().into();
        Id {
            left: value.clone(),
            right: value,
            ty: FINITE_TY.clone_ty(), // TODO: this...
            lt,
        }
    }
    /// Get the identity type for comparison between two values of the same type
    pub fn try_new(left: ValId, right: ValId) -> Result<Id, Error> {
        if left.ty() != right.ty() {
            return Err(Error::TypeMismatch);
        }
        let lt = left.lcr(&right)?.cloned_region().into();
        Ok(Id {
            left,
            right,
            lt,
            ty: FINITE_TY.clone_ty(), //TODO: this...
        })
    }
}

/*
/// A proof of identity for a set of values
///
/// Values of this type can only be constructed where the type of the values is of kind `#set`, implying identity is a mere proposition.
/// In this case `IdSet` is *always* a mere proposition.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct IdSet {
    /// The values being compared, or the single-value making up the identity set
    id_set: Either<ValSet, ValId>,
    /// The type of this identity type
    ty: TypeId,
    /// The lifetime of this identity type
    lt: Lifetime,
}
*/

/// The reflexivity axiom
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Refl {
    /// The base value
    value: ValId,
    /// The type of this invocation
    ///
    /// Can be either `Id` or `IdSet`
    ty: TypeId,
    /// The lifetime of this invocation
    ///
    /// For now always static, but left in for future-compatibility
    lt: Lifetime,
}

impl Refl {
    /// Create a new instance of the reflexivity axiom on a given `ValId`
    #[inline]
    pub fn refl(_value: ValId) -> Refl {
        unimplemented!("Refl construction, as Id is not a type yet")
    }
}
