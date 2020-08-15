/*!
Proofs of identity and equivalence.
*/
use crate::eval::Substitute;
use crate::eval::{Apply, EvalCtx};
use crate::lifetime::{Lifetime, LifetimeBorrow, Live};
use crate::region::Regional;
use crate::typing::{universe::FINITE_TY, Type, Typed};
use crate::value::{Error, TypeId, TypeRef, UniverseId, ValId};
use crate::{lifetime_region, substitute_to_valid};
use std::convert::TryInto;
//use either::Either;

/// The identity type family
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct IdFamily;

/// A proof of identity for two values
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Id {
    /// The left value being compared
    left: ValId,
    /// The right value being compared
    right: ValId,
    /// The type of this identity type
    ty: UniverseId,
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
            ty: FINITE_TY.clone(), // TODO: this...
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
            ty: FINITE_TY.clone(), //TODO: this...
        })
    }
}

impl Typed for Id {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
    #[inline]
    fn is_kind(&self) -> bool {
        false
    }
}

impl Live for Id {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.lt.borrow_lifetime()
    }
}

lifetime_region!(Id);

impl Apply for Id {}

impl Substitute for Id {
    #[inline]
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<Id, Error> {
        Ok(Id {
            left: self.left.substitute(ctx)?,
            right: self.right.substitute(ctx)?,
            lt: ctx.evaluate_lt(&self.lt)?,
            ty: self
                .ty
                .substitute(ctx)?
                .try_into()
                .map_err(|_| Error::TypeMismatch)?,
        })
    }
}

//substitute_to_valid!(Id);

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
