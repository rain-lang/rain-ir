/*!
`rain` values
*/
use crate::util::hash_cache::Cache;
use crate::{debug_from_display, enum_convert, forv, pretty_display};
use lazy_static::lazy_static;
use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use triomphe::{Arc, ArcBorrow};

pub mod expr;
pub mod lifetime;
pub mod primitive;
pub mod tuple;
pub mod typing;
pub mod universe;

use expr::Sexpr;
use lifetime::{LifetimeBorrow, Live, Parameter};
use primitive::Unit;
use tuple::{Product, Tuple};
use universe::Universe;

lazy_static! {
    /// The global `rain` value cache
    pub static ref VALUE_CACHE: Cache<NormalValue> = Cache::new();
}

/// A reference-counted, hash-consed `rain` value
#[derive(Clone, Eq)]
#[repr(transparent)]
pub struct ValId(Arc<NormalValue>);

impl Deref for ValId {
    type Target = Arc<NormalValue>;
    #[inline]
    fn deref(&self) -> &Arc<NormalValue> {
        &self.0
    }
}

impl From<NormalValue> for ValId {
    #[inline]
    fn from(value: NormalValue) -> ValId {
        ValId(VALUE_CACHE.cache(value))
    }
}

impl From<Arc<NormalValue>> for ValId {
    #[inline]
    fn from(value: Arc<NormalValue>) -> ValId {
        ValId(VALUE_CACHE.cache(value))
    }
}

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

/// A reference to a `rain` value
#[derive(Copy, Clone, Eq)]
pub struct ValRef<'a>(ArcBorrow<'a, NormalValue>);

impl PartialEq for ValRef<'_> {
    #[inline]
    fn eq(&self, other: &ValRef) -> bool {
        ArcBorrow::ptr_eq(&self.0, &other.0)
    }
}

impl Hash for ValRef<'_> {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.deref(), hasher)
    }
}

debug_from_display!(ValId);
pretty_display!(ValId, s, fmt  => write!(fmt, "{}", s.deref()));
debug_from_display!(ValRef<'_>);
pretty_display!(ValRef<'_>, s, fmt  => write!(fmt, "{}", s.deref()));

/// A reference-counted, hash-consed `rain` type
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TypeId(ValId);

impl Deref for TypeId {
    type Target = ValId;
    #[inline]
    fn deref(&self) -> &ValId {
        &self.0
    }
}

impl TypeId {
    /// Get this `TypeId` as a `ValId`
    #[inline]
    pub fn as_valid(&self) -> &ValId {
        &self
    }
}

/// A reference to a `rain` type
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct TypeRef<'a>(ValRef<'a>);

impl<'a> Deref for TypeRef<'a> {
    type Target = ValRef<'a>;
    #[inline]
    fn deref(&self) -> &ValRef<'a> {
        &self.0
    }
}

debug_from_display!(TypeId);
pretty_display!(TypeId, s, fmt => write!(fmt, "{}", s.deref()));
debug_from_display!(TypeRef<'_>);
pretty_display!(TypeRef<'_>, s, fmt => write!(fmt, "{}", s.deref()));

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
        forv! {
            match (value) {
                v => v.into(),
            }
        }
    }
}

impl Borrow<ValueEnum> for NormalValue {
    #[inline]
    fn borrow(&self) -> &ValueEnum {
        &self.0
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
    // ValueEnum injection:
    impl Injection<ValueEnum> for Sexpr {
        match
            other if *other == () => Ok(Sexpr::unit()),
            other => Ok(Sexpr::singleton(ValId::from(other))),
    }
    impl Injection<ValueEnum> for Parameter {}
    impl Injection<ValueEnum> for Tuple {
        match
            other if *other == () => Ok(Tuple::unit()),
    }
    impl Injection<ValueEnum> for Product {
        match
            other if *other == Unit => Ok(Product::unit_ty()),
    }
    impl Injection<ValueEnum> for Universe {}

    // NormalValue injection.
    impl Injection<NormalValue> for Sexpr {
        as ValueEnum,
        match
            other if *other == () => Ok(Sexpr::unit()),
            other => Ok(Sexpr::singleton(ValId::from(other))),
    }
    impl Injection<NormalValue> for Parameter { as ValueEnum, }
    impl Injection<NormalValue> for Tuple {
        as ValueEnum,
        match
            other if *other == () => Ok(Tuple::unit()),
    }
    impl Injection<NormalValue> for Product { as ValueEnum, } // No need to check for unit due to normalization!
    impl Injection<NormalValue> for Universe { as ValueEnum, }
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
    (match ($v:expr) { $i:ident => $e:expr, }) => {
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
            s => s.lifetime(),
        })
    }
}

/// Implement `ValId: From<T>` using `NormalValue: From<T>`
#[macro_export]
macro_rules! normal_valid {
    ($T:ty) => {
        impl From<$T> for $crate::value::ValId {
            #[inline]
            fn from(v: $T) -> $crate::value::ValId {
                $crate::value::NormalValue::from(v).into()
            }
        }
    };
}

normal_valid!(ValueEnum);
normal_valid!(Sexpr);
normal_valid!(Tuple);
normal_valid!(Product);
normal_valid!(Universe);

/// Implement `From<T>` for TypeId using the `From<T>` implementation of `ValId`, in effect
/// asserting that a type's values are all `rain` types
#[macro_use]
macro_rules! impl_to_type {
    ($T:ty) => {
        impl From<$T> for crate::value::TypeId {
            fn from(v: $T) -> crate::value::TypeId {
                crate::value::TypeId(crate::value::ValId::from(v))
            }
        }
    };
}

impl_to_type!(Product);
impl_to_type!(Universe);

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Formatter};

    impl PrettyPrint for ValueEnum {
        fn prettyprint(
            &self,
            printer: &mut PrettyPrinter,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            forv! {
                match (self) { v => v.prettyprint(printer, fmt), }
            }
        }
    }

    impl PrettyPrint for ValId {
        #[inline]
        fn prettyprint(
            &self,
            _printer: &mut PrettyPrinter,
            _fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            unimplemented!()
        }
    }

    impl PrettyPrint for ValRef<'_> {
        #[inline]
        fn prettyprint(
            &self,
            _printer: &mut PrettyPrinter,
            _fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            unimplemented!()
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

    impl PrettyPrint for TypeRef<'_> {
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
