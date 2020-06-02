/*!
Boolean types and logical operations
*/

use crate::prettyprinter::tokens::*;
use crate::value::{
    lifetime::{LifetimeBorrow, Live},
    typing::{Type, Typed},
    universe::FINITE_TY,
    TypeRef, UniverseRef, ValId, Value, VarId,
};
use crate::{debug_from_display, quick_pretty};
use lazy_static::lazy_static;

/// The type of booleans
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Bool;

lazy_static! {
    pub static ref BOOL_TY: VarId<Bool> = VarId::direct_new(Bool);
}

debug_from_display!(Bool);
quick_pretty!(Bool, "{}", KEYWORD_BOOL);

impl Typed for Bool {
    #[inline]
    fn ty(&self) -> TypeRef {
        FINITE_TY.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        true
    }
}

impl Value for Bool {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!("Bool has no dependencies (asked for dependency #{})", ix)
    }
}

impl Type for Bool {
    #[inline]
    fn is_universe(&self) -> bool {
        false
    }
    #[inline]
    fn universe(&self) -> UniverseRef {
        FINITE_TY.borrow_var()
    }
}

impl Live for Bool {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::default()
    }
}

impl Typed for bool {
    #[inline]
    fn ty(&self) -> TypeRef {
        BOOL_TY.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        false
    }
}

impl Live for bool {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::default()
    }
}

impl Value for bool {
    #[inline]
    fn no_deps(&self) -> usize {
        0
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        panic!("Boolean #{} has no dependencies (asked for dependency #{})", self, ix)
    }
}

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use crate::prettyprinter::{
        tokens::{KEYWORD_FALSE, KEYWORD_TRUE},
        PrettyPrint, PrettyPrinter,
    };
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for bool {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            _printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            match self {
                true => write!(fmt, "{}", KEYWORD_TRUE),
                false => write!(fmt, "{}", KEYWORD_FALSE),
            }
        }
    }
}
