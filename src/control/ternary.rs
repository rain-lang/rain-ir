/*!
A ternary operation
*/
use crate::function::pi::Pi;
use crate::lifetime::{Lifetime, LifetimeBorrow, Live};
use crate::lifetime_region;
use crate::typing::Typed;
use crate::value::{Error, TypeRef, ValId, VarId};

/// A ternary operation
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Ternary {
    /// The type of this ternary operation
    ty: VarId<Pi>,
    /// The lifetime of this ternary operation
    lt: Lifetime,
    /// The first branch of this ternary operation
    low: ValId,
    /// The second branch of this ternary operation
    high: ValId,
}

impl Ternary {
    /// Construct conditional ternary operation with the smallest possible type
    #[inline]
    pub fn conditional(high: ValId, low: ValId) -> Result<Ternary, Error> {
        use crate::primitive::logical::unary_region;
        let high_ty = high.ty();
        let low_ty = low.ty();
        let lt = (low.lifetime() & high.lifetime())?;
        let ty = if high_ty == low_ty {
            Pi::try_new(high_ty.clone_ty(), unary_region(), lt.clone())?.into()
        } else {
            unimplemented!("Dependently typed conditional: {} or {}", high, low);
        };
        Ok(Ternary { ty, lt, low, high })
    }
    /// Get the type of this ternary operation
    #[inline]
    pub fn get_ty(&self) -> &VarId<Pi> {
        &self.ty
    }
    /// Get the first branch of this ternary operation
    #[inline]
    pub fn low(&self) -> &ValId {
        &self.low
    }
    /// Get the second branch of this ternary operation
    #[inline]
    pub fn high(&self) -> &ValId {
        &self.high
    }
}

impl Typed for Ternary {
    #[inline]
    fn is_ty(&self) -> bool {
        false
    }
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
}

impl Live for Ternary {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.lt.borrow_lifetime()
    }
}

lifetime_region!(Ternary);
