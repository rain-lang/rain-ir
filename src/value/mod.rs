/*!
`rain` values
*/
use crate::{debug_from_display, display_pretty};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use triomphe::Arc;

pub mod expr;
pub mod lifetime;
pub mod primitive;
pub mod tuple;

use expr::Sexpr;
use lifetime::{LifetimeBorrow, Live, Parameter};
use tuple::{Product, Tuple};

/// A reference-counted, hash-consed `rain` value
#[derive(Clone, Eq)]
pub struct ValId(Arc<ValueEnum>);

impl PartialEq for ValId {
    #[inline]
    fn eq(&self, other: &ValId) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Hash for ValId {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.deref(), hasher)
    }
}

debug_from_display!(ValId);
display_pretty!(ValId, "{}", self.deref());

/// A reference-counted, hash-consed `rain` type
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TypeId(ValId);

debug_from_display!(TypeId);
display_pretty!(TypeId, "{}", self.deref());

/// An enumeration of possible `rain` values
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum ValueEnum {
    /// An S-expression
    Sexpr(Sexpr),
    /// A parameter
    Parameter(Parameter),
    /// A tuple of `rain` values
    Tuple(Tuple),
    /// A finite Cartesian product of `rain` types, at least some of which are distinct.
    Product(Product),
}

/// Perform an action for each variant of `ValueEnum`. Add additional match arms, if desired.
#[macro_export]
macro_rules! forv {
    (
        match ($v:expr) {
            $(if $p:pat => $m:expr,)*
            else $i:ident => $e:expr
        }
    ) => {
        #[allow(unreachable_patterns)]
        match $v {
            $($p:pat => $m:expr,)*
            ValueEnum::Sexpr($i) => $e,
            ValueEnum::Parameter($i) => $e,
            ValueEnum::Tuple($i) => $e,
            ValueEnum::Product($i) => $e
        }
    };
    (match ($v:expr) { $i:ident => $e:expr }) => {
        forv! {
            match ($v) {
                else $i => $e
            }
        }
    };
}


debug_from_display!(ValueEnum);
display_pretty!(ValueEnum, "TODO");

impl Live for ValueEnum {
    fn lifetime(&self) -> LifetimeBorrow {
        forv!(match (self) { s => s.lifetime() })
    }
}

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Formatter};

    impl PrettyPrint for ValueEnum {
        fn prettyprint(
            &self,
            _printer: &mut PrettyPrinter,
            _fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            unimplemented!()
        }
    }

    impl PrettyPrint for ValId {
        fn prettyprint(
            &self,
            _printer: &mut PrettyPrinter,
            _fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            unimplemented!()
        }
    }

    impl PrettyPrint for TypeId {
        fn prettyprint(
            &self,
            printer: &mut PrettyPrinter,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            self.0.prettyprint(printer, fmt)
        }
    }
}
