/*!
The `rain` type system
*/
use super::{NormalValue, PrivateValue, TypeId, TypeRef, UniverseRef, ValId, Value, ValueEnum};
use crate::{debug_from_display, pretty_display};
use ref_cast::RefCast;
use std::borrow::Borrow;
use std::convert::TryFrom;
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

impl Typed for TypeValue {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.deref().ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        self.deref().is_ty()
    }
}

impl Value for TypeValue {
    #[inline]
    fn no_deps(&self) -> usize {
        self.deref().no_deps()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        self.deref().get_dep(ix)
    }
}

impl Type for TypeValue {
    #[inline]
    fn universe(&self) -> UniverseRef {
        match self.borrow() {
            ValueEnum::Universe(u) => u.universe(),
            ValueEnum::Product(p) => p.universe(),
            ValueEnum::Parameter(_p) => unimplemented!(),
            _ => panic!("Impossible!"),
        }
    }
    #[inline]
    fn is_universe(&self) -> bool {
        match self.borrow() {
            ValueEnum::Universe(u) => u.is_universe(),
            ValueEnum::Product(p) => p.is_universe(),
            ValueEnum::Parameter(_p) => unimplemented!(),
            _ => panic!("Impossible!"),
        }
    }
}

impl From<TypeValue> for TypeId {
    fn from(_ty: TypeValue) -> TypeId {
        panic!("Dummy!")
    }
}

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

impl TryFrom<NormalValue> for TypeValue {
    type Error = NormalValue;
    #[inline]
    fn try_from(value: NormalValue) -> Result<TypeValue, NormalValue> {
        if value.is_ty() {
            Ok(TypeValue(value.0))
        } else {
            Err(value)
        }
    }
}

impl<'a> TryFrom<&'a NormalValue> for &'a TypeValue {
    type Error = &'a NormalValue;
    #[inline]
    fn try_from(value: &'a NormalValue) -> Result<&'a TypeValue, &'a NormalValue> {
        if value.is_ty() {
            Ok(RefCast::ref_cast(&value.0))
        } else {
            Err(value)
        }
    }
}

impl<'a> From<&'a TypeValue> for &'a NormalValue {
    fn from(value: &'a TypeValue) -> &'a NormalValue {
        RefCast::ref_cast(&value.0)
    }
}

impl Borrow<NormalValue> for TypeValue {
    fn borrow(&self) -> &NormalValue {
        self.into()
    }
}

impl<'a> From<&'a TypeValue> for &'a ValueEnum {
    fn from(value: &'a TypeValue) -> &'a ValueEnum {
        &(value.0).0
    }
}

impl Borrow<ValueEnum> for TypeValue {
    fn borrow(&self) -> &ValueEnum {
        self.into()
    }
}

impl TryFrom<ValueEnum> for TypeValue {
    type Error = ValueEnum;
    #[inline]
    fn try_from(value: ValueEnum) -> Result<TypeValue, ValueEnum> {
        if value.is_ty() {
            Ok(TypeValue(NormalValue::from(value).0))
        } else {
            Err(value)
        }
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
