/*!
`rain` expressions
*/
use super::{
    lifetime::{Lifetime, LifetimeBorrow, Live},
    primitive::UNIT_TY,
    typing::Typed,
    TypeId, TypeRef, ValId, Value,
};
use crate::{debug_from_display, pretty_display};
use smallvec::{smallvec, SmallVec};
use std::ops::Deref;

/// The size of a small S-expression
pub const SMALL_SEXPR_SIZE: usize = 3;

/// The argument-vector of an S-expression
pub type SexprArgs = SmallVec<[ValId; SMALL_SEXPR_SIZE]>;

/// An S-expression
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Sexpr {
    /// The arguments of this S-expression
    args: SexprArgs,
    /// The (cached) lifetime of this S-expression
    lifetime: Lifetime,
    /// The (cached) type of this S-expression
    ty: TypeId,
}

debug_from_display!(Sexpr);
pretty_display!(Sexpr, "(...)");

impl Sexpr {
    /// Create an S-expression corresponding to the unit value
    pub fn unit() -> Sexpr {
        Sexpr {
            args: SexprArgs::new(),
            lifetime: Lifetime::default(),
            ty: UNIT_TY.as_ty().clone(),
        }
    }
    /// Create an S-expression corresponding to a singleton value
    pub fn singleton(value: ValId) -> Sexpr {
        let ty = value.ty().clone_ty();
        Sexpr {
            args: smallvec![value],
            lifetime: Lifetime::default(),
            ty,
        }
    }
}

impl Live for Sexpr {
    fn lifetime(&self) -> LifetimeBorrow {
        self.lifetime.borrow_lifetime()
    }
}

impl Typed for Sexpr {
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
}

impl Value for Sexpr {
    #[inline]
    fn no_deps(&self) -> usize {
        self.len()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        &self[ix]
    }
}

impl Deref for Sexpr {
    type Target = SexprArgs;
    fn deref(&self) -> &SexprArgs {
        &self.args
    }
}

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Formatter};

    impl PrettyPrint for Sexpr {
        fn prettyprint(
            &self,
            _printer: &mut PrettyPrinter,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            write!(fmt, "(")?;
            for _value in self.iter() {
                unimplemented!()
            }
            write!(fmt, ")")
        }
    }
}
