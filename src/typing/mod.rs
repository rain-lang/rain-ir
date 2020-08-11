/*!
The `rain` type system
*/
use super::{
    eval::{Apply, EvalCtx, Substitute},
    lifetime::{Lifetime, LifetimeBorrow, Live},
    region::{RegionBorrow, Regional},
    value::{Error, NormalValue, TypeId, TypeRef, UniverseRef, ValId, Value, ValueData, ValueEnum},
};
use crate::{debug_from_display, pretty_display};
use std::borrow::Borrow;
use std::convert::{TryFrom, TryInto};
use std::ops::Deref;

mod kind;
pub use kind::*;

/// A trait implemented by `rain` values with a type
pub trait Typed {
    /// Compute the type of this `rain` value
    fn ty(&self) -> TypeRef;
    /// Check whether this `rain` value is a type
    fn is_ty(&self) -> bool;
}

/// A trait implemented by `rain` values which are a type
pub trait Type: Value {
    /// Convert this type into a `TypeId`
    fn into_ty(self) -> TypeId {
        self.into_val().coerce()
    }
    /// Get the universe of this type
    fn universe(&self) -> UniverseRef;
    /// Get whether this type is a universe
    fn is_universe(&self) -> bool;
    /// Get whether this type is affine
    fn is_affine(&self) -> bool;
    /// Get whether this type is relevant
    fn is_relevant(&self) -> bool;
    /// Get whether this type is linear
    #[inline]
    fn is_linear(&self) -> bool {
        self.is_affine() && self.is_relevant()
    }
    /// Get whether this type is substructural
    #[inline]
    fn is_substruct(&self) -> bool {
        self.is_affine() || self.is_relevant()
    }
    /// Apply this type to a set of arguments, yielding a result type and lifetime
    fn apply_ty(&self, args: &[ValId]) -> Result<(Lifetime, TypeId), Error>
    where
        Self: Clone,
    {
        self.apply_ty_in(args, &mut None)
    }
    /// Apply this type to a set of arguments, yielding a result type and lifetime
    fn apply_ty_in(
        &self,
        args: &[ValId],
        _ctx: &mut Option<EvalCtx>,
    ) -> Result<(Lifetime, TypeId), Error>
    where
        Self: Clone,
    {
        if args.is_empty() {
            Ok((self.lifetime().clone_lifetime(), self.clone().into_ty()))
        } else {
            Err(Error::NotAFunction)
        }
    }
}

/// A value guaranteed to be a type
#[derive(Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct TypeValue(NormalValue);

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

impl Live for TypeValue {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.deref().lifetime()
    }
}

impl Regional for TypeValue {
    #[inline]
    fn region(&self) -> Option<RegionBorrow> {
        self.deref().region()
    }
}

impl Substitute for TypeId {
    #[inline]
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<TypeId, Error> {
        let v: ValId = self.as_val().substitute(ctx)?;
        v.try_into().map_err(|_| Error::NotATypeError)
    }
}

impl Substitute<ValId> for TypeValue {
    #[inline]
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<ValId, Error> {
        self.deref().substitute(ctx)
    }
}

impl Apply for TypeValue {}

impl Value for TypeValue {
    #[inline]
    fn no_deps(&self) -> usize {
        self.deref().no_deps()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        self.deref().get_dep(ix)
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        self.into()
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

impl ValueData for TypeValue {}

impl Type for TypeValue {
    #[inline]
    fn universe(&self) -> UniverseRef {
        match self.borrow() {
            ValueEnum::Universe(u) => u.universe(),
            ValueEnum::Product(p) => p.universe(),
            ValueEnum::Parameter(_p) => unimplemented!("Parameter universe getter"),
            ValueEnum::Sexpr(_s) => unimplemented!(),
            ValueEnum::BoolTy(b) => b.universe(),
            ValueEnum::Pi(p) => p.universe(),
            ValueEnum::Finite(f) => f.universe(),
            u => panic!(
                "Impossible (TypeValue::universe): TypeValue({}) is not a type",
                u
            ),
        }
    }
    #[inline]
    fn is_universe(&self) -> bool {
        match self.borrow() {
            ValueEnum::Universe(u) => u.is_universe(),
            ValueEnum::Product(p) => p.is_universe(),
            ValueEnum::Parameter(_p) => unimplemented!("Parameter universe check"),
            ValueEnum::Sexpr(_s) => unimplemented!(),
            ValueEnum::BoolTy(b) => b.is_universe(),
            ValueEnum::Pi(p) => p.is_universe(),
            ValueEnum::Finite(f) => f.is_universe(),
            u => panic!(
                "Impossible (TypeValue::is_universe): TypeValue({}) is not a type",
                u
            ),
        }
    }
    #[inline]
    fn is_affine(&self) -> bool {
        match self.borrow() {
            ValueEnum::Universe(u) => u.is_affine(),
            ValueEnum::Product(p) => p.is_affine(),
            ValueEnum::Parameter(_p) => unimplemented!(),
            ValueEnum::Sexpr(_s) => unimplemented!(),
            ValueEnum::BoolTy(b) => b.is_affine(),
            ValueEnum::Pi(p) => p.is_affine(),
            ValueEnum::Finite(f) => f.is_affine(),
            u => panic!(
                "Impossible (TypeValue::is_affine): TypeValue({}) is not a type",
                u
            ),
        }
    }

    #[inline]
    fn is_relevant(&self) -> bool {
        match self.borrow() {
            ValueEnum::Universe(u) => u.is_relevant(),
            ValueEnum::Product(p) => p.is_relevant(),
            ValueEnum::Parameter(_p) => unimplemented!(),
            ValueEnum::Sexpr(_s) => unimplemented!(),
            ValueEnum::BoolTy(b) => b.is_relevant(),
            ValueEnum::Pi(p) => p.is_relevant(),
            ValueEnum::Finite(f) => f.is_relevant(),
            u => panic!(
                "Impossible (TypeValue::is_relevant): TypeValue({}) is not a type",
                u
            ),
        }
    }

    #[inline]
    fn is_linear(&self) -> bool {
        match self.borrow() {
            ValueEnum::Universe(u) => u.is_linear(),
            ValueEnum::Product(p) => p.is_linear(),
            ValueEnum::Parameter(_p) => unimplemented!(),
            ValueEnum::Sexpr(_s) => unimplemented!(),
            ValueEnum::BoolTy(b) => b.is_linear(),
            ValueEnum::Pi(p) => p.is_linear(),
            ValueEnum::Finite(f) => f.is_linear(),
            u => panic!(
                "Impossible (TypeValue::is_linear): TypeValue({}) is not a type",
                u
            ),
        }
    }

    #[inline]
    fn is_substruct(&self) -> bool {
        match self.borrow() {
            ValueEnum::Universe(u) => u.is_substruct(),
            ValueEnum::Product(p) => p.is_substruct(),
            ValueEnum::Parameter(_p) => unimplemented!(),
            ValueEnum::Sexpr(_s) => unimplemented!(),
            ValueEnum::BoolTy(b) => b.is_substruct(),
            ValueEnum::Pi(p) => p.is_substruct(),
            ValueEnum::Finite(f) => f.is_substruct(),
            u => panic!(
                "Impossible (TypeValue::is_substruct): TypeValue({}) is not a type",
                u
            ),
        }
    }
    #[inline]
    fn apply_ty_in(
        &self,
        args: &[ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<(Lifetime, TypeId), Error> {
        match self.borrow() {
            ValueEnum::Universe(u) => u.apply_ty_in(args, ctx),
            ValueEnum::Product(p) => p.apply_ty_in(args, ctx),
            ValueEnum::Parameter(_p) => unimplemented!(),
            ValueEnum::Sexpr(_s) => unimplemented!(),
            ValueEnum::BoolTy(b) => b.apply_ty_in(args, ctx),
            ValueEnum::Pi(p) => p.apply_ty_in(args, ctx),
            ValueEnum::Finite(f) => f.apply_ty_in(args, ctx),
            u => panic!(
                "Impossible (TypeValue::apply_ty): TypeValue({}) is not a type",
                u
            ),
        }
    }
}

impl From<TypeValue> for ValId {
    fn from(ty: TypeValue) -> ValId {
        ValId::<()>::direct_new(ty)
    }
}

impl Deref for TypeValue {
    type Target = NormalValue;
    fn deref(&self) -> &NormalValue {
        &self.0
    }
}

impl From<TypeValue> for NormalValue {
    fn from(ty: TypeValue) -> NormalValue {
        ty.0
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
            Ok(TypeValue(value))
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
            let cast = unsafe { &*(value as *const NormalValue as *const TypeValue) };
            Ok(cast)
        } else {
            Err(value)
        }
    }
}

impl<'a> From<&'a TypeValue> for &'a NormalValue {
    fn from(value: &'a TypeValue) -> &'a NormalValue {
        &value.0
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
            Ok(TypeValue(NormalValue::from(value)))
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
