/*!
Phi nodes, describing general recursion
*/

use crate::eval::{Apply, EvalCtx, Substitute};
use crate::lifetime::{LifetimeBorrow, Live};
use crate::typing::Typed;
use crate::value::{
    arr::{ValArr, ValSet},
    tuple::Product,
    Error, NormalValue, TypeRef, ValId, Value, ValueEnum, VarId,
};
use crate::{debug_from_display, enum_convert, pretty_display, substitute_to_valid};

/// A phi node, representing mutual recursion
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Phi {
    /// The tuple of recursively defined objects in this node
    ///
    /// This should be nonempty, and all objects should reside in the same region, namely the defining region
    values: ValArr,
    /// The dependencies of this node
    deps: ValSet,
    /// The type of this phi node as a value
    ty: VarId<Product>,
}

impl Live for Phi {
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::STATIC
    }
}

impl Typed for Phi {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        false
    }
    #[inline]
    fn is_kind(&self) -> bool {
        false
    }
}

impl Substitute for Phi {
    fn substitute(&self, _ctx: &mut EvalCtx) -> Result<Phi, Error> {
        unimplemented!()
    }
}

impl Value for Phi {
    #[inline]
    fn no_deps(&self) -> usize {
        self.deps.len()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        &self.deps[ix]
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Phi(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

impl Apply for Phi {
    //TODO: again, pretty important, right? I mean, Turing-completeness is kind of a big deal...
}

impl From<Phi> for NormalValue {
    fn from(p: Phi) -> NormalValue {
        NormalValue::assert_normal(ValueEnum::Phi(p))
    }
}

substitute_to_valid!(Phi);
debug_from_display!(Phi);
pretty_display!(Phi, "{}{{ ... }}", crate::tokens::KEYWORD_PHI);
enum_convert! {
    impl InjectionRef<ValueEnum> for Phi {}
    impl TryFrom<NormalValue> for Phi { as ValueEnum, }
    impl TryFromRef<NormalValue> for Phi { as ValueEnum, }
}

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for Phi {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            _printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            write!(fmt, "UNIMPLEMENTED!")
        }
    }
}
