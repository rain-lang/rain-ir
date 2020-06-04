/*!
Pi types
*/
use crate::value::{
    eval::Apply,
    lifetime::Parametrized,
    lifetime::{LifetimeBorrow, Live},
    typing::{Type, Typed},
    TypeId, TypeRef, UniverseId, UniverseRef, ValId, Value,
};
use crate::{debug_from_display, pretty_display};

/// A pi type
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Pi {
    /// The result of this pi type
    result: Parametrized<TypeId>,
    /// The type of this pi type
    ty: UniverseId,
}

impl Typed for Pi {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
}

impl Live for Pi {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.result.lifetime()
    }
}

impl Apply for Pi {
    //TODO: allow limited pi-application?
}

impl Value for Pi {
    #[inline]
    fn no_deps(&self) -> usize {
        self.result.deps().len()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        &self.result.deps()[ix]
    }
}

impl Type for Pi {
    #[inline]
    fn is_universe(&self) -> bool {
        false
    }
    #[inline]
    fn universe(&self) -> UniverseRef {
        self.ty.borrow_var()
    }
}

debug_from_display!(Pi);
pretty_display!(Pi, "#pi|...| {...}");

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for Pi {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            _printer: &mut PrettyPrinter<I>,
            _fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            unimplemented!()
        }
    }
}
