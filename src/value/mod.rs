/*!
`rain` values
*/
use crate::{debug_from_display, enum_convert, forv, pretty_display};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use triomphe::Arc;

pub mod expr;
pub mod lifetime;
pub mod primitive;
pub mod tuple;
pub mod universe;

use expr::Sexpr;
use lifetime::{LifetimeBorrow, Live, Parameter};
use tuple::{Product, Tuple};
use universe::Universe;

/// A reference-counted, hash-consed `rain` value
#[derive(Clone, Eq)]
pub struct ValId(Arc<NormalValue>);

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
pretty_display!(ValId, s, fmt  => write!(fmt, "{}", s.deref()));

/// A reference-counted, hash-consed `rain` type
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TypeId(ValId);

debug_from_display!(TypeId);
pretty_display!(TypeId, s, fmt => write!(fmt, "{}", s.deref()));

/// A normalized `rain` value
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct NormalValue(ValueEnum);

impl Deref for NormalValue {
    type Target = ValueEnum;
    fn deref(&self) -> &ValueEnum {
        &self.0
    }
}

impl From<ValueEnum> for NormalValue {
    #[inline]
    fn from(value: ValueEnum) -> NormalValue {
        /*
        forv! {
            match (value) {
                v => unimplemented!()
            }
        }
        */
        let _ = value;
        unimplemented!()
    }
}

impl From<NormalValue> for ValueEnum {
    #[inline]
    fn from(normal: NormalValue) -> ValueEnum {
        normal.0
    }
}

debug_from_display!(NormalValue);
pretty_display!(NormalValue, s, fmt => write!(fmt, "{}", s.deref()));

/// A trait implemented by `rain` values
pub trait Value: Into<NormalValue> + Into<ValueEnum> {}

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
    /// A typing universe
    Universe(Universe),
}

enum_convert! {
    impl Injection<ValueEnum> for Sexpr {}
    impl Injection<ValueEnum> for Parameter {}
    impl Injection<ValueEnum> for Tuple {}
    impl Injection<ValueEnum> for Product {}
    //TODO: unit normalization
    impl TryFrom<NormalValue> for Sexpr { match ValueEnum::Sexpr, }
    impl TryFrom<NormalValue> for Parameter { match ValueEnum::Parameter, }
    impl TryFrom<NormalValue> for Tuple { match ValueEnum::Tuple, }
    impl TryFrom<NormalValue> for Product { match ValueEnum::Product, }
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
            ValueEnum::Product($i) => $e,
            ValueEnum::Universe($i) => $e,
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
pretty_display!(ValueEnum, v, fmt => forv! {
    match (v) { v => write!(fmt, "{}", v) }
});

impl Live for ValueEnum {
    fn lifetime(&self) -> LifetimeBorrow {
        forv!(match (self) {
            s => s.lifetime()
        })
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
        #[inline]
        fn prettyprint(
            &self,
            printer: &mut PrettyPrinter,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            self.deref().prettyprint(printer, fmt)
        }
    }

    impl PrettyPrint for TypeId {
        #[inline]
        fn prettyprint(
            &self,
            printer: &mut PrettyPrinter,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            self.0.prettyprint(printer, fmt)
        }
    }

    impl PrettyPrint for NormalValue {
        #[inline]
        fn prettyprint(
            &self,
            printer: &mut PrettyPrinter,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            self.0.prettyprint(printer, fmt)
        }
    }
}
