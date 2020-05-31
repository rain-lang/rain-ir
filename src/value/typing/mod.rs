/*!
The `rain` type system
*/
use super::{NormalValue, PrivateValue, TypeId, TypeRef, UniverseRef, Value, ValueEnum};
use crate::{debug_from_display, pretty_display};
use ref_cast::RefCast;
use std::ops::Deref;

/// A trait implemented by `rain` values with a type
pub trait Typed {
    /// Compute the type of this `rain` value
    fn ty(&self) -> TypeRef;
    /// Check whether this `rain` value is a type
    fn is_ty(&self) -> bool;
}

/// A trait implemented by `rain` values which are a type
pub trait Type: Into<TypeId> + Value {
    /// Get the universe of this type
    fn universe(&self) -> UniverseRef;
    /// Get whether this type is a universe
    fn is_universe(&self) -> bool;
}

/// A value guaranteed to be a type
#[derive(Eq, PartialEq, Hash, RefCast)]
#[repr(transparent)]
pub struct TypeValue(PrivateValue);

debug_from_display!(TypeValue);
pretty_display!(TypeValue, s, fmt => write!(fmt, "{}", s.deref()));

impl Deref for TypeValue {
    type Target = NormalValue;
    fn deref(&self) -> &NormalValue {
        RefCast::ref_cast(&self.0)
    }
}

impl From<TypeValue> for NormalValue {
    fn from(ty: TypeValue) -> NormalValue {
        NormalValue(ty.0)
    }
}

impl From<TypeValue> for ValueEnum {
    fn from(ty: TypeValue) -> ValueEnum {
        (ty.0).0
    }
}

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for TypeValue {
        #[inline]
        fn prettyprint<I: From<usize> + Display>(
            &self,
            printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            if let Some(name) = printer.lookup(self) {
                write!(fmt, "{}", name)
            } else {
                self.deref().prettyprint(printer, fmt)
            }
        }
    }
}
