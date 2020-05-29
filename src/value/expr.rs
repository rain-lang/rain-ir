/*!
`rain` expressions
*/
use super::{
    lifetime::{Lifetime, LifetimeBorrow, Live},
    TypeId, ValId,
};
use crate::{debug_from_display, pretty_display};
use smallvec::SmallVec;

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
    ///
    /// TODO: Optional?
    ty: TypeId,
}

debug_from_display!(Sexpr);
pretty_display!(Sexpr, "(...)");

impl Sexpr {
    /// Create an S-expression corresponding to the unit value
    pub fn unit() -> Sexpr {
        unimplemented!()
    }
    /// Create an S-expression corresponding to a singleton value
    pub fn singleton(_value: ValId) -> Sexpr {
        unimplemented!() // Needs typing...
    }
}

impl Live for Sexpr {
    fn lifetime(&self) -> LifetimeBorrow {
        self.lifetime.borrow_lifetime()
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
            _fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            unimplemented!()
        }
    }
}
