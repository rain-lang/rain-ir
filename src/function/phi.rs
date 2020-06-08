/*!
Phi nodes, representing mutual recursion
*/
use crate::eval::{Apply, EvalCtx, Substitute};
use crate::lifetime::{Lifetime, LifetimeBorrow, Live, Region};
use crate::typing::Typed;
use crate::value::{tuple::Product, Error, TypeRef, ValId, Value, VarId};
use crate::{debug_from_display, pretty_display, substitute_to_valid};
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
}

impl Apply for Phi {
    //TODO: again, pretty important, right? I mean, Turing-completeness is kind of a big deal...
}

substitute_to_valid!(Phi);
debug_from_display!(Phi);
pretty_display!(Phi, "{}{{ ... }}", prettyprinter::tokens::KEYWORD_PHI);

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
