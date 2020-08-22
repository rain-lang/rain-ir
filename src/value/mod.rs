/*!
`rain` values
*/
use crate::control::{phi::Phi, ternary::Ternary};
use crate::eval::{Application, Apply, EvalCtx, Substitute};
use crate::function::{lambda::Lambda, pi::Pi};
use crate::primitive::{
    bits::{Bits, BitsTy},
    finite::{Finite, Index},
    logical::{Bool, Logical},
};
use crate::proof::identity::{Id, IdFamily, PathInd, Refl};
use crate::region::{Parameter, RegionBorrow, Regional};
use crate::typing::primitive::{Fin, Prop, Set};
use crate::typing::{IsKind, IsRepr, IsType, IsUniverse, Typed};
use crate::{debug_from_display, forv, pretty_display};
use dashcache::{DashCache, GlobalCache};
use elysees::{Arc, ArcBorrow};
use lazy_static::lazy_static;
use ref_cast::RefCast;
use std::borrow::Borrow;
use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;

pub mod arr;
pub mod expr;
pub mod predicate;
pub mod sum;
pub mod tuple;

use arr::ValSet;
use expr::Sexpr;
use predicate::Is;
use tuple::{Product, Tuple};

mod error;
mod valid_impl;
pub use error::*;
pub use valid_impl::*;

// Basic value type declarations:

/// A `rain` value, optionally asserted to satisfy a predicate `P`
#[repr(transparent)]
pub struct ValId<P = ()> {
    ptr: Arc<NormalValue>,
    variant: PhantomData<P>,
}

/// A borrowed `rain` value, optionally guaranteed to satisfy a given predicate `P`
#[repr(transparent)]
pub struct ValRef<'a, P = ()> {
    ptr: ArcBorrow<'a, NormalValue>,
    variant: PhantomData<P>,
}

/// An enumeration of possible `rain` values
///
/// The `ValueEnum` is the central data-structure defining the `rain` intermediate representation:
/// it lays out all the possible kinds of nodes which can make up part of the `rain`-graph.
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
    /// A mere proposition
    Prop(Prop),
    /// A finite type
    Fin(Fin),
    /// An n-set
    Set(Set),
    /// The type of Bits
    BitsTy(BitsTy),
    /// A bitset value
    Bits(Bits),
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
    /// A ternary operation
    Ternary(Ternary),
    /// A phi node
    Phi(Phi),
    /// Logical operations on booleans
    Logical(Logical),
    /// Identity types
    Id(Id),
    /// The `refl` constructor for identity types
    Refl(Refl),
    /// A family of identity types
    IdFamily(IdFamily),
    /// An instance of the path induction axiom
    PathInd(PathInd),
}

// Common value type aliases:

/// A `rain` type
pub type TypeId = ValId<IsType>;

/// A `rain` type reference
pub type TypeRef<'a> = ValRef<'a, IsType>;

/// A `rain` value known to be a type
pub type TypeValue = NormalValue<IsType>;

/// A `rain` kind
pub type KindId = ValId<IsKind>;

/// A `rain` kind reference
pub type KindRef<'a> = ValRef<'a, IsKind>;

/// A `rain` value known to be a kind
pub type KindValue = NormalValue<IsKind>;

/// A `rain` representation
pub type ReprId = ValId<IsRepr>;

/// A `rain` representation reference
pub type ReprRef<'a> = ValRef<'a, IsRepr>;

/// A `rain` value known to be a representation
pub type ReprValue = NormalValue<IsRepr>;

/// A `rain` universe
pub type UniverseId = ValId<IsUniverse>;

/// A `rain` universe reference
pub type UniverseRef<'a> = ValRef<'a, IsUniverse>;

/// A `rain` value known to be a universe
pub type UniverseValue = NormalValue<IsUniverse>;

/// A value guaranteed to be a certain `ValueEnum` variant (may not be an actual variant)
pub type VarId<V> = ValId<Is<V>>;

/// A borrowed value guaranteed to be a certain `ValueEnum` variant (may not be an actual variant)
pub type VarRef<'a, V> = ValRef<'a, Is<V>>;

// The `Value` trait:

/// A trait implemented by all `rain` values
pub trait Value: Sized + Typed + Apply + Substitute<ValId> + Regional {
    /// Get the number of dependencies of this value
    ///
    /// Note that this does *not* include the type: as all `rain` values have exactly one type, this is counted
    /// separately from normal dependencies (but is important in, e.g., lifetime considerations!)
    fn no_deps(&self) -> usize;
    /// Get a given dependency of this value
    ///
    /// The result of this function is unspecified if the `dep` index is out of bounds, though it will always either
    /// return a valid [`&ValId`](ValId) or panic. This function must never panic if the `dep` index is in bounds.
    fn get_dep(&self, dep: usize) -> &ValId;
    /// Get the dependencies of this value
    #[inline]
    fn deps(&self) -> &Deps<Self> {
        RefCast::ref_cast(self)
    }
    /// Clone the dependency-set of this value
    ///
    /// This returns an owned `ValSet` of the dependencies of this value, i.e. a sorted, de-duplicated dependency array.
    #[inline]
    fn clone_depset(&self) -> ValSet {
        self.deps().iter().cloned().collect()
    }
    /// Convert a value into a `NormalValue`
    fn into_norm(self) -> NormalValue;
    /// Convert a value into a `ValueEnum`
    ///
    /// Note that the return value of this function is *not necessarily normalized*! For that, use [`self.into_norm()`](Value::into_norm)
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
    /// Apply this value to a set of arguments, if possible
    #[inline]
    fn applied(&self, args: &[ValId]) -> Result<ValId, Error>
    where
        Self: Clone,
    {
        let application = self.curried(args)?;
        let (rest, success) = application.valid_to_success(self, args);
        debug_assert!(
            rest.is_empty(),
            "Incomplete currying: {:?} left, got {:?}",
            rest,
            success
        );
        Ok(success)
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
    /// Convert a value into a `KindId` if it is a kind, otherwise return it
    #[inline]
    fn try_into_kind(self) -> Result<KindId, Self> {
        if self.is_kind() {
            Ok(self.into_val().coerce())
        } else {
            Err(self)
        }
    }
    /// Convert a value into a `ReprId` if it is a representation, otherwise return it
    #[inline]
    fn try_into_repr(self) -> Result<ReprId, Self> {
        if self.is_repr() {
            Ok(self.into_val().coerce())
        } else {
            Err(self)
        }
    }
    /// Convert a value into a `UniverseId` if it is a universe, otherwise return it
    #[inline]
    fn try_into_universe(self) -> Result<UniverseId, Self> {
        if self.is_universe() {
            Ok(self.into_val().coerce())
        } else {
            Err(self)
        }
    }
}

/// A trait implemented by non-pointer `rain` values
pub trait ValueData: Value {}

// Utilities:

/// The address of a value
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct ValAddr(pub usize);

impl From<ValAddr> for usize {
    #[inline(always)]
    fn from(val: ValAddr) -> usize {
        val.0
    }
}

impl From<usize> for ValAddr {
    #[inline(always)]
    fn from(addr: usize) -> ValAddr {
        ValAddr(addr)
    }
}

impl<P> From<*const NormalValue<P>> for ValAddr {
    #[inline(always)]
    fn from(addr: *const NormalValue<P>) -> ValAddr {
        ValAddr(addr as usize)
    }
}

impl From<*const ValueEnum> for ValAddr {
    #[inline(always)]
    fn from(addr: *const ValueEnum) -> ValAddr {
        ValAddr(addr as usize)
    }
}

impl<P> From<&'_ NormalValue<P>> for ValAddr {
    #[inline(always)]
    fn from(addr: &NormalValue<P>) -> ValAddr {
        ValAddr(addr as *const _ as usize)
    }
}

impl From<&'_ ValueEnum> for ValAddr {
    #[inline(always)]
    fn from(addr: &ValueEnum) -> ValAddr {
        ValAddr(addr as *const _ as usize)
    }
}

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

impl<P> Substitute for NormalValue<P> {
    #[inline]
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<NormalValue<P>, Error> {
        self.value.substitute(ctx).map(NormalValue::<()>::coerce)
    }
}

impl<P> Substitute<ValId> for NormalValue<P> {
    #[inline]
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<ValId, Error> {
        self.value.substitute(ctx)
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

/// A normalized `rain` value, asserted to satisfy a given predicate
#[derive(Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct NormalValue<P = ()> {
    value: ValueEnum,
    predicate: PhantomData<P>,
}

impl<P> Clone for NormalValue<P> {
    #[inline(always)]
    fn clone(&self) -> NormalValue<P> {
        NormalValue {
            value: self.value.clone(),
            predicate: PhantomData,
        }
    }
}

impl<P> NormalValue<P> {
    /// Coerce this value to one guaranteed to satisfy a different predicate
    #[inline(always)]
    pub(crate) fn coerce<Q>(self) -> NormalValue<Q> {
        NormalValue {
            value: self.value,
            predicate: PhantomData,
        }
    }
    /// Coerce a reference to this value to one guaranteed to satisfy a different predicate
    #[inline(always)]
    pub(crate) fn coerce_ref<Q>(&self) -> &NormalValue<Q> {
        unsafe { &*(self as *const _ as *const NormalValue<Q>) }
    }
    /// Get this normal value as a plain normal value
    #[inline(always)]
    pub fn as_norm(&self) -> &NormalValue {
        self.coerce_ref()
    }
    /// Get this `NormalValue` as a `ValueEnum`
    #[inline(always)]
    pub fn as_enum(&self) -> &ValueEnum {
        &self.value
    }
}

impl NormalValue {
    /// Assert a given value is a normal value
    #[inline(always)]
    pub(crate) fn assert_normal(value: ValueEnum) -> NormalValue {
        NormalValue {
            value,
            predicate: PhantomData,
        }
    }
    /*
    /// Assert a reference to a given value is a reference to a normal value
    pub(crate) fn assert_ref(value: &ValueEnum) -> &NormalValue {
        unsafe { &*(value as *const ValueEnum as *const NormalValue) }
    }
    */
}

impl<P> Deref for NormalValue<P> {
    type Target = ValueEnum;
    #[inline(always)]
    fn deref(&self) -> &ValueEnum {
        &self.value
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
        &self.value
    }
}

impl From<NormalValue> for ValueEnum {
    #[inline]
    fn from(normal: NormalValue) -> ValueEnum {
        normal.value
    }
}

impl<P> Typed for NormalValue<P> {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.value.ty()
    }
    #[inline]
    fn kind(&self) -> KindRef {
        self.value.kind()
    }
    #[inline]
    fn repr(&self) -> Option<ReprRef> {
        self.value.repr()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        self.value.is_ty()
    }
    #[inline]
    fn is_kind(&self) -> bool {
        self.value.is_kind()
    }
    #[inline]
    fn is_repr(&self) -> bool {
        self.value.is_repr()
    }
    #[inline]
    fn is_universe(&self) -> bool {
        self.value.is_universe()
    }
    #[inline]
    fn kind_level(&self) -> usize {
        self.value.kind_level()
    }
}

impl<P> Apply for NormalValue<P> {
    #[inline]
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        self.value.apply_in(args, ctx)
    }
}

impl<P> Value for NormalValue<P> {
    #[inline]
    fn no_deps(&self) -> usize {
        self.value.no_deps()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        self.value.get_dep(ix)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.coerce()
    }
}

impl<P> Regional for NormalValue<P> {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.value.region()
    }
    #[inline]
    fn depth(&self) -> usize {
        self.value.depth()
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
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
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
            ValueEnum::Prop($i) => $e,
            ValueEnum::Fin($i) => $e,
            ValueEnum::Set($i) => $e,
            ValueEnum::BoolTy($i) => $e,
            ValueEnum::Bool($i) => $e,
            ValueEnum::Finite($i) => $e,
            ValueEnum::Index($i) => $e,
            ValueEnum::Pi($i) => $e,
            ValueEnum::Lambda($i) => $e,
            ValueEnum::Ternary($i) => $e,
            ValueEnum::Phi($i) => $e,
            ValueEnum::Logical($i) => $e,
            ValueEnum::Id($i) => $e,
            ValueEnum::Refl($i) => $e,
            ValueEnum::IdFamily($i) => $e,
            ValueEnum::BitsTy($i) => $e,
            ValueEnum::Bits($i) => $e,
            ValueEnum::PathInd($i) => $e,
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

impl Regional for ValueEnum {
    #[inline]
    fn region(&self) -> RegionBorrow {
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
    fn kind(&self) -> KindRef {
        forv!(match (self) {
            s => s.kind(),
        })
    }
    #[inline]
    fn repr(&self) -> Option<ReprRef> {
        forv!(match (self) {
            s => s.repr(),
        })
    }
    #[inline]
    fn is_ty(&self) -> bool {
        forv!(match (self) {
            s => s.is_ty(),
        })
    }
    #[inline]
    fn is_kind(&self) -> bool {
        forv!(match (self) {
            s => s.is_kind(),
        })
    }
    #[inline]
    fn is_repr(&self) -> bool {
        forv!(match (self) {
            s => s.is_repr(),
        })
    }
    #[inline]
    fn is_universe(&self) -> bool {
        forv!(match (self) {
            s => s.is_universe(),
        })
    }
    #[inline]
    fn kind_level(&self) -> usize {
        forv!(match (self) {
            s => s.kind_level(),
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
normal_valid!(Prop);
normal_valid!(Fin);
normal_valid!(Set);
normal_valid!(Bool);
normal_valid!(bool); //TODO
normal_valid!(Finite); //TODO: unit + empty?
normal_valid!(Index); //TODO: unit?
normal_valid!(Pi);
normal_valid!(Lambda);
normal_valid!(Parameter);
normal_valid!(Phi);
normal_valid!(Logical);
normal_valid!(Ternary);
normal_valid!(Id);
normal_valid!(Refl);
normal_valid!(IdFamily);
normal_valid!(BitsTy);
normal_valid!(Bits);
normal_valid!(PathInd);

/// Implement `From<T>` for TypeValue using the `From<T>` implementation of `NormalValue`, in effect
/// asserting that a type's values are all `rain` types
#[macro_use]
macro_rules! impl_to_type {
    ($T:ty) => {
        impl From<$T> for crate::value::TypeId {
            fn from(v: $T) -> crate::value::TypeId {
                v.try_into_ty().expect("Infallible!")
            }
        }
    };
}

impl_to_type!(Product);
impl_to_type!(Set);
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
