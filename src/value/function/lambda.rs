/*!
Lambda functions
*/
use super::pi::Pi;
use crate::value::{
    eval::Apply,
    lifetime::Live,
    lifetime::{LifetimeBorrow, Parametrized, Region},
    typing::Typed,
    TypeRef, ValId, Value, VarId,
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

impl Lambda {
    /// Create a new lambda function from a parametrized `ValId`
    pub fn new(result: Parametrized<ValId>) -> Lambda {
        let ty = VarId::from(Pi::ty(&result));
        Lambda {
            result,
            ty
        }
    }
    /// Attempt to create a new lambda function from a region and value
    pub fn try_new(value: ValId, region: Region) -> Result<Lambda, ()> {
        Ok(Self::new(Parametrized::try_new(value, region)?))
    }
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

impl Apply for Lambda {
    //TODO: this, this one is pretty important, no?
}

impl Value for Lambda {
    #[inline]
    fn no_deps(&self) -> usize {
        self.result.deps().len()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        &self.result.deps()[ix]
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
