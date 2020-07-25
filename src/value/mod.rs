/*!
`rain` values
*/
use crate::eval::{Application, Apply, EvalCtx, Substitute};
use crate::function::{gamma::Gamma, lambda::Lambda, phi::Phi, pi::Pi};
use crate::lifetime::{Lifetime, LifetimeBorrow, Live};
use crate::primitive::{
    finite::{Finite, Index},
    logical::{Bool, Logical},
};
use crate::region::{Parameter, RegionBorrow, Regional};
use crate::typing::{Type, TypeValue, Typed};
use crate::{debug_from_display, forv, pretty_display};
use dashcache::{DashCache, GlobalCache};
use elysees::{Arc, ArcBorrow};
use lazy_static::lazy_static;
use ref_cast::RefCast;
use std::borrow::Borrow;
use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::Hash;
use std::ops::Deref;

pub mod arr;
pub mod expr;
pub mod predicate;
pub mod tuple;
pub mod universe;

use expr::Sexpr;
use predicate::Is;
use tuple::{Product, Tuple};
use universe::Universe;
use arr::ValSet;

mod error;
mod valid_impl;
pub use error::*;
pub use valid_impl::*;

// Basic value type declarations:

/// A `rain` value, optionally asserted to satisfy a predicate `P`
#[repr(transparent)]
pub struct ValId<P = ()> {
    ptr: Arc<NormalValue>,
    variant: std::marker::PhantomData<P>,
}

/// A borrowed `rain` value, optionally guaranteed to satisfy a given predicate `P`
#[repr(transparent)]
pub struct ValRef<'a, P = ()> {
    ptr: ArcBorrow<'a, NormalValue>,
    variant: std::marker::PhantomData<P>,
}

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
    /// The type of booleans
    BoolTy(Bool),
    /// A boolean value
    Bool(bool),
    /// A finite type
    Finite(Finite),
    /// An index into a finite type
    Index(Index),
    /// A pi type
    Pi(Pi),
    /// A lambda function
    Lambda(Lambda),
    /// A gamma node
    Gamma(Gamma),
    /// A phi node
    Phi(Phi),
    /// Logical operations on booleans
    Logical(Logical),
}

// Common value type aliases:

/// A `rain` type
pub type TypeId = VarId<TypeValue>;

/// A `rain` type reference
pub type TypeRef<'a> = VarRef<'a, TypeValue>;

/// A reference-counted pointer to a value guaranteed to be a typing universe
pub type UniverseId = VarId<Universe>;

/// A pointer to a value guaranteed to be a typing universe
pub type UniverseRef<'a> = VarRef<'a, Universe>;

/// A value guaranteed to be a certain `ValueEnum` variant (may not be an actual variant)
pub type VarId<V> = ValId<Is<V>>;

/// A borrowed value guaranteed to be a certain `ValueEnum` variant (may not be an actual variant)
pub type VarRef<'a, V> = ValRef<'a, Is<V>>;

// The `Value` trait:

/// A trait implemented by `rain` values
pub trait Value: Sized + Typed + Live + Apply + Substitute<ValId> + Regional {
    /// Get the number of dependencies of this value
    fn no_deps(&self) -> usize;
    /// Get a given dependency of this value
    fn get_dep(&self, dep: usize) -> &ValId;
    /// Get the dependencies of this value
    #[inline]
    fn deps(&self) -> &Deps<Self> {
        RefCast::ref_cast(self)
    }
    /// Clone the dependency-set of this value
    #[inline]
    fn clone_depset(&self) -> ValSet {
        self.deps().iter().cloned().collect()
    }
    /// Convert a value into a `NormalValue`
    fn into_norm(self) -> NormalValue;
    /// Convert a value into a `ValueEnum`
    #[inline]
    fn into_enum(self) -> ValueEnum {
        self.into_norm().into()
    }
    /// Convert a value into a `ValId`
    #[inline]
    fn into_val(self) -> ValId {
        self.into_norm().into()
    }
    /// Convert a value into a `VarId`
    #[inline]
    fn into_var(self) -> VarId<Self> {
        self.into_val().coerce()
    }
    /// Convert a value into a `TypeId`, if it is a type, otherwise return it
    #[inline]
    fn try_into_ty(self) -> Result<TypeId, Self> {
        if self.is_ty() {
            Ok(self.into_val().coerce())
        } else {
            Err(self)
        }
    }
    /// Get the cast target lifetime for a given lifetime
    fn cast_target_lt(&self, lt: Lifetime) -> Result<Lifetime, Error> {
        self.lifetime().as_lifetime() * lt
    }
    /// Get the cast target type for a given type
    fn cast_target_ty(&self, ty: TypeId) -> Result<TypeId, Error> {
        if ty != self.ty() {
            //TODO: this...
            return Err(Error::TypeMismatch);
        }
        Ok(ty)
    }
    /// Cast a value to a given type and lifetime
    #[inline]
    fn cast(self, ty: Option<TypeId>, lt: Option<Lifetime>) -> Result<ValId, Error> {
        if ty.is_none() && lt.is_none() {
            return Ok(self.into_val());
        }
        let lt = if let Some(lt) = lt {
            self.cast_target_lt(lt)?
        } else {
            self.lifetime().clone_lifetime()
        };
        let ty = if let Some(ty) = ty {
            self.cast_target_ty(ty)?
        } else {
            self.ty().clone_ty()
        };
        if lt == self.lifetime() && ty == self.ty() {
            return Ok(self.into_val());
        }
        let val = self.into_val();
        Ok(NormalValue(Sexpr::cast_singleton(val, lt, ty).into_enum()).into())
    }
    /// Cast a value to a given lifetime
    #[inline]
    fn cast_lt(self, lt: Lifetime) -> Result<ValId, Error> {
        self.cast(None, Some(lt))
    }
    /// Cast a value to a given type
    #[inline]
    fn cast_ty(self, ty: TypeId) -> Result<ValId, Error> {
        self.cast(Some(ty), None)
    }
}

/// A trait implemented by non-pointer `rain` values
pub trait ValueData: Value {}

// Utilities:

/// The dependencies of a value
#[derive(Debug, Copy, Clone, RefCast)]
#[repr(transparent)]
pub struct Deps<V>(pub V);

impl<V: Value> Deps<V> {
    /// The number of dependencies of this value
    pub fn len(&self) -> usize {
        self.0.no_deps()
    }
    /// Check whether this value has no dependencies
    pub fn is_empty(&self) -> bool {
        self.0.no_deps() == 0
    }
    /// Iterate over the dependencies of this value
    pub fn iter<'a>(
        &'a self,
    ) -> impl Iterator<Item = &'a ValId> + DoubleEndedIterator + ExactSizeIterator + 'a {
        (0..self.len()).map(move |ix| self.0.get_dep(ix))
    }
}

// Implementation:

impl Substitute for NormalValue {
    #[inline]
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<NormalValue, Error> {
        self.deref().substitute(ctx)
    }
}

impl Substitute<ValId> for NormalValue {
    #[inline]
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<ValId, Error> {
        self.deref().substitute(ctx)
    }
}

impl Substitute for ValueEnum {
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<ValueEnum, Error> {
        forv! { match(self) {
            v => v.substitute(ctx),
        } }
    }
}

impl Substitute<NormalValue> for ValueEnum {
    #[inline]
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<NormalValue, Error> {
        self.substitute(ctx).map(|val: ValueEnum| val.into())
    }
}

impl Substitute<ValId> for ValueEnum {
    #[inline]
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<ValId, Error> {
        forv! { match(self) {
            v => v.substitute(ctx),
        } }
    }
}

/// A normalized `rain` value
#[derive(Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct NormalValue(pub(crate) ValueEnum);

impl NormalValue {
    /*
    /// Assert a reference to a given value is a reference to a normal value
    pub(crate) fn assert_ref(value: &ValueEnum) -> &NormalValue {
        unsafe { &*(value as *const ValueEnum as *const NormalValue) }
    }
    */
}

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

impl Typed for NormalValue {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.deref().ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        self.deref().is_ty()
    }
}

impl Apply for NormalValue {
    #[inline]
    fn apply_in<'a>(&self, args: &'a [ValId], ctx: &mut Option<EvalCtx>) -> Result<Application<'a>, Error> {
        self.0.apply_in(args, ctx)
    }
}

impl Value for NormalValue {
    #[inline]
    fn no_deps(&self) -> usize {
        self.deref().no_deps()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        self.deref().get_dep(ix)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self
    }
}

impl Live for NormalValue {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.deref().lifetime()
    }
}

impl Regional for NormalValue {
    #[inline]
    fn region(&self) -> Option<RegionBorrow> {
        self.deref().region()
    }
    #[inline]
    fn depth(&self) -> usize {
        self.deref().depth()
    }
}

debug_from_display!(NormalValue);
pretty_display!(NormalValue, s, fmt => write!(fmt, "{}", s.deref()));

impl<V: Value> std::ops::Index<usize> for Deps<V> {
    type Output = ValId;
    fn index(&self, ix: usize) -> &ValId {
        self.0.get_dep(ix)
    }
}

impl Apply for ValueEnum {
    #[inline]
    fn apply_in<'a>(&self, args: &'a [ValId], ctx: &mut Option<EvalCtx>) -> Result<Application<'a>, Error> {
        forv! {match (self) {
            v => v.apply_in(args, ctx),
        }}
    }
}

impl Value for ValueEnum {
    fn no_deps(&self) -> usize {
        forv! {
            match(self) {
                v => v.no_deps(),
            }
        }
    }
    fn get_dep(&self, ix: usize) -> &ValId {
        forv! {
            match(self) {
                v => v.get_dep(ix),
            }
        }
    }
    fn into_enum(self) -> ValueEnum {
        self
    }
    fn into_norm(self) -> NormalValue {
        self.into()
    }
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
            ValueEnum::BoolTy($i) => $e,
            ValueEnum::Bool($i) => $e,
            ValueEnum::Finite($i) => $e,
            ValueEnum::Index($i) => $e,
            ValueEnum::Pi($i) => $e,
            ValueEnum::Lambda($i) => $e,
            ValueEnum::Gamma($i) => $e,
            ValueEnum::Phi($i) => $e,
            ValueEnum::Logical($i) => $e,
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
    match (v) { v => write!(fmt, "{}", v), }
});

impl Live for ValueEnum {
    fn lifetime(&self) -> LifetimeBorrow {
        forv!(match (self) {
            s => s.lifetime(),
        })
    }
}

impl Regional for ValueEnum {
    #[inline]
    fn region(&self) -> Option<RegionBorrow> {
        forv!(match (self) {
            s => s.region(),
        })
    }
    #[inline]
    fn depth(&self) -> usize {
        forv!(match (self) {
            s => s.depth(),
        })
    }
}

impl Typed for ValueEnum {
    #[inline]
    fn ty(&self) -> TypeRef {
        forv!(match (self) {
            s => s.ty(),
        })
    }
    #[inline]
    fn is_ty(&self) -> bool {
        forv!(match (self) {
            s => s.is_ty(),
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
                v.into_val()
            }
        }
    };
}

normal_valid!(ValueEnum);
normal_valid!(Sexpr);
normal_valid!(Tuple);
normal_valid!(Product);
normal_valid!(Universe);
normal_valid!(Bool);
normal_valid!(bool); //TODO
normal_valid!(Finite); //TODO: unit + empty?
normal_valid!(Index); //TODO: unit?
normal_valid!(Pi);
normal_valid!(Lambda);
normal_valid!(Parameter);
normal_valid!(Gamma);
normal_valid!(Phi);
normal_valid!(Logical);

/// Implement `From<T>` for TypeValue using the `From<T>` implementation of `NormalValue`, in effect
/// asserting that a type's values are all `rain` types
#[macro_use]
macro_rules! impl_to_type {
    ($T:ty) => {
        impl From<$T> for crate::value::TypeValue {
            fn from(v: $T) -> crate::typing::TypeValue {
                crate::typing::TypeValue::try_from(crate::value::NormalValue::from(v))
                    .expect("Impossible")
            }
        }
        impl From<$T> for crate::value::TypeId {
            fn from(v: $T) -> crate::value::TypeId {
                v.try_into_ty().expect("Infallible!")
            }
        }
    };
}

impl_to_type!(Product);
impl_to_type!(Universe);
impl_to_type!(Bool);
impl_to_type!(Finite);
impl_to_type!(Pi);

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Formatter};

    impl PrettyPrint for ValueEnum {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            forv! {
                match (self) { v => v.prettyprint(printer, fmt), }
            }
        }
    }

    impl PrettyPrint for NormalValue {
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
