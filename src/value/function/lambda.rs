/*!
Lambda functions
*/
use super::pi::Pi;
use crate::value::{
    lifetime::Live,
    lifetime::{LifetimeBorrow, Parametrized},
    typing::Typed,
    TypeRef, ValId, VarId,
};
use crate::{debug_from_display, pretty_display};

/// A lambda function
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Lambda {
    /// The result of this lambda function
    result: Parametrized<ValId>,
    /// The type of this lambda function
    ty: VarId<Pi>,
}

impl Typed for Lambda {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        false
    }
}

impl Live for Lambda {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.result.lifetime()
    }
}

debug_from_display!(Lambda);
pretty_display!(Lambda, "#lambda |...| {...}");

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for Lambda {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            _printer: &mut PrettyPrinter<I>,
            _fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            unimplemented!()
        }
    }
}
