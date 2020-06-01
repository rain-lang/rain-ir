/*!
Boolean types and logical operations
*/

use crate::prettyprinter::tokens::*;
use crate::{debug_from_display, quick_pretty};
use crate::value::{typing::Typed, universe::FINITE_TY, TypeRef, ValId, Value, lifetime::{Live, LifetimeBorrow}};

/// The type of booleans
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Bool;

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

impl Live for Bool {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        LifetimeBorrow::default()
    }
}