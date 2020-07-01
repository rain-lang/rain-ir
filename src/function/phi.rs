/*!
Phi nodes, representing mutual recursion
*/
use crate::eval::{Apply, EvalCtx, Substitute};
use crate::lifetime::{Lifetime, LifetimeBorrow, Live};
use crate::region::Region;
use crate::typing::Typed;
use crate::value::{tuple::Product, Error, NormalValue, TypeRef, ValId, Value, ValueEnum, VarId};
use crate::{
    debug_from_display, enum_convert, lifetime_region, pretty_display, substitute_to_valid,
};
use smallvec::SmallVec;

/// The size of a small set of mutually recursive values
pub const SMALL_PHI_SIZE: usize = 2;

/// A phi node, representing mutual recursion
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Phi {
    /// The region corresponding to the recursively defined objects in this node
    region: Region,
    /// The tuple of recursively defined objects in this node
    values: SmallVec<[ValId; SMALL_PHI_SIZE]>,
    /// The dependencies of this node
    deps: Box<[ValId]>,
    /// The lifetime of this phi node as a value
    lifetime: Lifetime,
    /// The type of this phi node as a value
    ty: VarId<Product>,
}

impl Live for Phi {
    fn lifetime(&self) -> LifetimeBorrow {
        self.lifetime.borrow_lifetime()
    }
}

lifetime_region!(Phi);

impl Typed for Phi {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
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
        NormalValue(ValueEnum::Phi(p))
    }
}

substitute_to_valid!(Phi);
debug_from_display!(Phi);
pretty_display!(Phi, "{}{{ ... }}", prettyprinter::tokens::KEYWORD_PHI);
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
