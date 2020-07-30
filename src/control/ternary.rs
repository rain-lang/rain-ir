/*!
A ternary operation
*/
use crate::eval::{Application, Apply, EvalCtx};
use crate::function::pi::Pi;
use crate::lifetime::{Lifetime, LifetimeBorrow, Live};
use crate::lifetime_region;
use crate::primitive::logical::BOOL_TY;
use crate::typing::Typed;
use crate::value::{Error, TypeId, TypeRef, ValId, Value, ValueEnum, VarId};

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
    /// Get the parameter type type of this ternary operation
    #[inline]
    pub fn param_ty(&self) -> &TypeId {
        &self.ty.param_tys()[0]
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
    /// Get whether this ternary node is constant. Should always be `false` for a normalized node!
    #[inline]
    pub fn is_const(&self) -> bool {
        self.low == self.high
    }
    /// Get the ternary kind of this node
    #[inline]
    pub fn ternary_kind(&self) -> TernaryKind {
        if *self.param_ty() == BOOL_TY.borrow_ty() {
            TernaryKind::Bool
        } else {
            panic!("Invalid ternary node: {:#?}", self)
        }
    }
}
/// Kinds of ternary node
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TernaryKind {
    /// A boolean branch
    Bool,
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

impl Apply for Ternary {
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        _ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        // Empty application
        if args.is_empty() {
            return Ok(Application::Complete(self.lt.clone(), self.ty().clone_ty()));
        }
        match self.ternary_kind() {
            TernaryKind::Bool => {
                if let ValueEnum::Bool(b) = args[0].as_enum() {
                    let rest = &args[1..];
                    if *b {
                        Ok(Application::Success(rest, self.high.clone()))
                    } else {
                        Ok(Application::Success(rest, self.low.clone()))
                    }
                } else {
                    if args[0].ty() == BOOL_TY.borrow_ty() {
                        unimplemented!("Unevaluated ternary nodes")
                    } else {
                        Err(Error::TypeMismatch)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::finite::Finite;

    #[test]
    fn basic_conditional_application() {
        let finite: VarId<Finite> = Finite(6).into();
        let high = finite.clone().ix(3).unwrap().into_val();
        let low = finite.ix(1).unwrap().into_val();
        let ternary = Ternary::conditional(high.clone(), low.clone()).unwrap();
        assert_eq!(
            ternary.apply(&[true.into()]).unwrap(),
            Application::Success(&[], high)
        );
        assert_eq!(
            ternary.apply(&[false.into()]).unwrap(),
            Application::Success(&[], low)
        );
    }
}
