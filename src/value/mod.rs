/*!
`rain` values
*/
use crate::eval::{Application, Apply, EvalCtx, Substitute};
use crate::function::{gamma::Gamma, lambda::Lambda, phi::Phi, pi::Pi};
use crate::lifetime::{LifetimeBorrow, Live};
use crate::primitive::{
    finite::{Finite, Index},
    logical::{Bool, Logical},
    Unit,
};
use crate::region::{Parameter, RegionBorrow, Regional};
use crate::typing::{Type, TypeValue, Typed};
use crate::util::{hash_cache::Cache, PrivateByAddr};
use crate::{debug_from_display, enum_convert, forv, pretty_display};
use fxhash::FxHashSet;
use lazy_static::lazy_static;
use ref_cast::RefCast;
use smallvec::SmallVec;
use std::borrow::Borrow;
use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::Hash;
use std::ops::{Deref, RangeBounds};
use triomphe::{Arc, ArcBorrow};

pub mod arr;
mod error;
pub mod expr;
pub mod predicate;
pub mod tuple;
pub mod universe;
use predicate::Is;

pub use error::*;
use expr::Sexpr;
use tuple::{Product, Tuple};
use universe::Universe;

lazy_static! {
    /// The global `rain` value cache
    pub static ref VALUE_CACHE: Cache<NormalValue> = Cache::new();
}

impl ValId {
    /// Directly construct a `ValId` from a `NormalValue`, deduplicating but not performing any other transformation/caching.
    /// Useful to prevent infinite regress in e.g. cached constructors for `()`
    #[inline]
    pub fn direct_new<V: Into<NormalValue>>(v: V) -> ValId {
        let norm = v.into();
        ValId {
            ptr: NormAddr::make(VALUE_CACHE.cache(norm), Private {}),
            variant: std::marker::PhantomData,
        }
    }
    /// Deduplicate an `Arc<NormalValue>` to yield a `ValId`
    #[inline]
    pub fn dedup(norm: Arc<NormalValue>) -> ValId {
        ValId {
            ptr: NormAddr::make(VALUE_CACHE.cache(norm), Private {}),
            variant: std::marker::PhantomData,
        }
    }
    /// Perform a substitution. Here to avoid code duplication during monomorphization
    pub fn substitute_impl(&self, ctx: &mut EvalCtx) -> Result<ValId, Error> {
        if let Some(value) = ctx.try_evaluate(self) {
            return Ok(value);
        }
        let result: ValId = self.deref().substitute(ctx)?;
        ctx.substitute(self.clone(), result.clone(), false)?;
        Ok(result)
    }
}

impl Deref for ValId {
    type Target = NormalValue;
    #[inline]
    fn deref(&self) -> &NormalValue {
        &self.ptr
    }
}

impl Borrow<NormalValue> for ValId {
    #[inline]
    fn borrow(&self) -> &NormalValue {
        &self.ptr
    }
}

impl Borrow<ValueEnum> for ValId {
    #[inline]
    fn borrow(&self) -> &ValueEnum {
        &self.ptr
    }
}

impl From<NormalValue> for ValId {
    #[inline]
    fn from(value: NormalValue) -> ValId {
        ValId {
            ptr: NormAddr::make(VALUE_CACHE.cache(value), Private {}),
            variant: std::marker::PhantomData,
        }
    }
}

impl From<Arc<NormalValue>> for ValId {
    #[inline]
    fn from(value: Arc<NormalValue>) -> ValId {
        ValId {
            ptr: NormAddr::make(VALUE_CACHE.cache(value), Private {}),
            variant: std::marker::PhantomData,
        }
    }
}

impl From<ValId> for ValueEnum {
    fn from(val: ValId) -> ValueEnum {
        val.as_enum().clone()
    }
}

impl From<ValId> for NormalValue {
    fn from(val: ValId) -> NormalValue {
        val.as_norm().clone()
    }
}

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
        self.substitute(ctx)
            .map(|v: ValueEnum| NormalValue::from(v))
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

impl<'a> Deref for ValRef<'a> {
    type Target = NormalValue;
    #[inline]
    fn deref(&self) -> &NormalValue {
        &self.ptr
    }
}

impl<P> Live for ValRef<'_, P> {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.as_norm().lifetime()
    }
}

impl<P> Regional for ValRef<'_, P> {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.as_norm().region()
    }
}

impl<P> Typed for ValRef<'_, P> {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.as_norm().ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        self.as_norm().is_ty()
    }
}

impl<P> Value for ValRef<'_, P> {
    #[inline]
    fn no_deps(&self) -> usize {
        self.as_norm().no_deps()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        self.as_norm().get_dep(ix)
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        self.as_enum().clone()
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.as_norm().clone()
    }
    #[inline]
    fn into_val(self) -> ValId {
        self.clone_val()
    }
}

impl From<ValRef<'_>> for ValueEnum {
    fn from(val: ValRef) -> ValueEnum {
        val.as_enum().clone()
    }
}

impl From<ValRef<'_>> for NormalValue {
    fn from(val: ValRef) -> NormalValue {
        val.as_norm().clone()
    }
}

impl From<ValRef<'_>> for ValId {
    fn from(val: ValRef) -> ValId {
        val.clone_val()
    }
}

impl Borrow<NormalValue> for ValRef<'_> {
    #[inline]
    fn borrow(&self) -> &NormalValue {
        &self.ptr
    }
}

impl Borrow<ValueEnum> for ValRef<'_> {
    #[inline]
    fn borrow(&self) -> &ValueEnum {
        &self.ptr
    }
}

impl<P> Substitute<ValId> for ValRef<'_, P> {
    #[inline]
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<ValId, Error> {
        self.clone_val().substitute(ctx)
    }
}

debug_from_display!(ValId);
pretty_display!(ValId, s, fmt  => write!(fmt, "{}", s.deref()));
debug_from_display!(ValRef<'_>);
pretty_display!(ValRef<'_>, s, fmt  => write!(fmt, "{}", s.deref()));

/// A `rain` type
pub type TypeId = VarId<TypeValue>;

/// A `rain` type reference
pub type TypeRef<'a> = VarRef<'a, TypeValue>;

/// A `rain` value, optionally asserted to satisfy a predicate `P`
#[derive(Hash, RefCast)]
#[repr(transparent)]
pub struct ValId<P = ()> {
    ptr: NormAddr,
    variant: std::marker::PhantomData<P>,
}

/// A value guaranteed to be a certain `ValueEnum` variant (may not be an actual variant)
pub type VarId<V> = ValId<Is<V>>;

/// A borrowed value guaranteed to be a certain `ValueEnum` variant (may not be an actual variant)
pub type VarRef<'a, V> = ValRef<'a, Is<V>>;

impl<'a, P, Q> PartialEq<ValRef<'a, P>> for ValId<Q> {
    fn eq(&self, other: &ValRef<'a, P>) -> bool {
        self.ptr == other.ptr
    }
}

impl<P, Q> PartialEq<ValId<P>> for ValId<Q> {
    fn eq(&self, other: &ValId<P>) -> bool {
        self.ptr == other.ptr
    }
}

impl<P> Eq for ValId<P> {}

impl<P> Clone for ValId<P> {
    #[inline]
    fn clone(&self) -> ValId<P> {
        ValId {
            ptr: self.ptr.clone(),
            variant: self.variant,
        }
    }
}

impl<P> ValId<P> {
    /// Get this `ValId<P>` as a `NormalValue`
    pub fn as_norm(&self) -> &NormalValue {
        self.ptr.deref()
    }
    /// Get this `ValId<P>` as a `ValueEnum`
    pub fn as_enum(&self) -> &ValueEnum {
        self.ptr.deref()
    }
    /// Get this `ValId<P>` as a `ValId`
    pub fn as_val(&self) -> &ValId {
        RefCast::ref_cast(&self.ptr)
    }
    /// Get the pointer behind this `ValId`
    #[inline]
    pub fn as_ptr(&self) -> *const NormalValue {
        self.as_norm() as *const NormalValue
    }
    /// Try to get this `ValId<P>` as a type
    #[inline]
    pub fn try_as_ty(&self) -> Result<&TypeId, &ValId<P>> {
        if self.is_ty() {
            Ok(self.coerce_ref())
        } else {
            Err(self)
        }
    }
    /// Borrow this `ValId<P>` as a `ValRef`
    pub fn borrow_val(&self) -> ValRef {
        ValRef {
            ptr: self.ptr.borrow_arc(),
            variant: std::marker::PhantomData,
        }
    }
    /// Borrow this `ValId<P>` as a `ValRef<P>`
    pub fn borrow_var(&self) -> ValRef<P> {
        ValRef {
            ptr: self.ptr.borrow_arc(),
            variant: self.variant,
        }
    }
    /// Coerce this `ValId` into another predicated value
    fn coerce<Q>(self) -> ValId<Q> {
        ValId {
            ptr: self.ptr,
            variant: std::marker::PhantomData,
        }
    }
    /// Coerce this `ValId` into a reference to another predicated value
    fn coerce_ref<Q>(&self) -> &ValId<Q> {
        RefCast::ref_cast(&self.ptr)
    }
}

impl<'a, V> VarId<V> {
    /// Directly construct a `VarId` from a `V`, deduplicating but not performing any other transformation/caching.
    /// Useful to prevent infinite regress in e.g. cached constructors for `()`
    #[inline]
    pub fn direct_new(v: V) -> VarId<V>
    where
        V: Into<NormalValue>,
    {
        let norm: NormalValue = v.into();
        VarId {
            ptr: NormAddr::make(VALUE_CACHE.cache(norm), Private {}),
            variant: std::marker::PhantomData,
        }
    }
    /// Get this `VarId` as a `TypeId`
    pub fn as_ty(&self) -> &TypeId
    where
        V: Type,
    {
        RefCast::ref_cast(&self.ptr)
    }
    /// Borrow this `VarId` as a `TypeRef`
    pub fn borrow_ty(&self) -> TypeRef
    where
        V: Type,
    {
        VarRef {
            ptr: self.ptr.borrow_arc(),
            variant: std::marker::PhantomData,
        }
    }
}

impl<V> From<VarId<V>> for ValId {
    fn from(v: VarId<V>) -> ValId {
        ValId {
            ptr: v.ptr,
            variant: std::marker::PhantomData,
        }
    }
}

impl<V> Deref for VarId<V>
where
    for<'a> &'a NormalValue: TryInto<&'a V>,
{
    type Target = V;
    fn deref(&self) -> &V {
        match self.ptr.deref().try_into() {
            Ok(r) => r,
            _ => panic!("Impossible: VarId is not asserted variant"),
        }
    }
}

impl<V> TryFrom<ValId> for VarId<V>
where
    for<'a> &'a NormalValue: TryInto<&'a V>,
{
    type Error = ValId;
    fn try_from(v: ValId) -> Result<VarId<V>, ValId> {
        if TryInto::<&V>::try_into(v.as_norm()).is_ok() {
            Ok(VarId {
                ptr: v.ptr,
                variant: std::marker::PhantomData,
            })
        } else {
            Err(v)
        }
    }
}

impl<V> From<V> for VarId<V>
where
    V: Into<ValId>,
    for<'a> &'a NormalValue: TryInto<&'a V>,
{
    fn from(val: V) -> VarId<V> {
        let valid: ValId = val.into();
        VarId {
            ptr: valid.ptr,
            variant: std::marker::PhantomData,
        }
    }
}

impl<P> Typed for ValId<P> {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.as_norm().ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        self.as_norm().is_ty()
    }
}

impl<P> Apply for ValId<P> {
    #[inline]
    fn do_apply<'a>(&self, args: &'a [ValId], inline: bool) -> Result<Application<'a>, Error> {
        self.ptr.do_apply(args, inline)
    }
}

impl<P> Substitute<ValId> for ValId<P> {
    #[inline]
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<ValId, Error> {
        self.as_val().substitute_impl(ctx)
    }
}

impl<P> Regional for ValId<P> {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.as_norm().region()
    }
}

impl<P> Live for ValId<P> {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.ptr.lifetime()
    }
}

impl<P> Value for ValId<P> {
    #[inline]
    fn no_deps(&self) -> usize {
        self.as_norm().no_deps()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        self.as_norm().get_dep(ix)
    }
    #[inline]
    fn into_enum(self) -> ValueEnum {
        self.as_enum().clone()
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.as_norm().clone()
    }
    #[inline]
    fn into_val(self) -> ValId {
        ValId {
            ptr: self.ptr,
            variant: std::marker::PhantomData,
        }
    }
}

impl<V: Value> From<VarId<V>> for ValueEnum {
    fn from(val: VarId<V>) -> ValueEnum {
        val.as_enum().clone()
    }
}

impl<V: Value> From<VarId<V>> for NormalValue {
    fn from(val: VarId<V>) -> NormalValue {
        val.as_norm().clone()
    }
}

impl<'a, V: Value> From<&'a VarId<V>> for &'a ValId {
    fn from(var: &'a VarId<V>) -> &'a ValId {
        RefCast::ref_cast(&var.ptr)
    }
}

/// A reference-counted pointer to a value guaranteed to be a typing universe
pub type UniverseId = VarId<Universe>;

/// A pointer to a value guaranteed to be a typing universe
pub type UniverseRef<'a> = VarRef<'a, Universe>;

/// A (*normalized, consed*) borrowed value, optionally guaranteed to satisfy a given predicate `P`
#[derive(Eq, Hash, RefCast)]
#[repr(transparent)]
pub struct ValRef<'a, P = ()> {
    ptr: NormRef<'a>,
    variant: std::marker::PhantomData<P>,
}

impl<'a, U, V> PartialEq<ValRef<'a, U>> for ValRef<'a, V> {
    fn eq(&self, other: &ValRef<'a, U>) -> bool {
        self.ptr == other.ptr
    }
}

impl<'a, U, V> PartialEq<ValId<U>> for ValRef<'a, V> {
    fn eq(&self, other: &ValId<U>) -> bool {
        self.ptr == other.ptr
    }
}

impl<'a, P> Clone for ValRef<'a, P> {
    #[inline]
    fn clone(&self) -> ValRef<'a, P> {
        ValRef {
            ptr: self.ptr,
            variant: self.variant,
        }
    }
}

impl<'a, P> Copy for ValRef<'a, P> {}

impl<'a, P> ValRef<'a, P> {
    /// Get this `ValRef<P>` as a `NormalValue`
    pub fn as_norm(&self) -> &'a NormalValue {
        self.ptr.get()
    }
    /// Get this `ValRef<P>` as a `ValueEnum`
    pub fn as_enum(&self) -> &'a ValueEnum {
        self.ptr.get()
    }
    /// Get this `ValRef<P>` as a `ValRef`
    pub fn as_val(&self) -> ValRef<'a> {
        ValRef {
            ptr: self.ptr,
            variant: std::marker::PhantomData,
        }
    }
    /// Clone this `VarRef` as a `ValId`
    pub fn clone_val(&self) -> ValId {
        ValId {
            ptr: self.ptr.clone_arc(),
            variant: std::marker::PhantomData,
        }
    }
}

impl<'a, V> VarRef<'a, V> {
    /// Get this `VarRef` as a `TypeRef`
    pub fn as_ty(&self) -> TypeRef<'a>
    where
        V: Type,
    {
        VarRef {
            ptr: self.ptr,
            variant: std::marker::PhantomData,
        }
    }
    /// Clone this `VarRef` as a `TypeId`
    pub fn clone_ty(&self) -> TypeId
    where
        V: Type,
    {
        self.as_ty().clone_var()
    }
    /// Clone this `VarRef` as a `VarId`
    pub fn clone_var(&self) -> VarId<V> {
        VarId {
            ptr: self.ptr.clone_arc(),
            variant: self.variant,
        }
    }
    /// Get the pointer behind this `VarRef`
    #[inline]
    pub fn as_ptr(&self) -> *const NormalValue {
        self.as_norm() as *const NormalValue
    }
}

impl<V> Deref for VarRef<'_, V>
where
    for<'a> &'a NormalValue: TryInto<&'a V>,
{
    type Target = V;
    fn deref(&self) -> &V {
        match self.ptr.deref().try_into() {
            Ok(r) => r,
            _ => panic!("Impossible: VarRef is not asserted variant"),
        }
    }
}

impl<V> Debug for VarId<V>
where
    V: Value,
{
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(self.as_norm(), fmt)
    }
}

impl<V> Debug for VarRef<'_, V>
where
    V: Value,
{
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(self.as_norm(), fmt)
    }
}

impl<V> Display for VarId<V>
where
    V: Value,
{
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        Display::fmt(self.as_norm(), fmt)
    }
}

impl<V> Display for VarRef<'_, V>
where
    V: Value,
{
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        Display::fmt(self.as_norm(), fmt)
    }
}

impl<'a, V: 'a> TryFrom<ValRef<'a>> for VarRef<'a, V>
where
    &'a NormalValue: TryInto<&'a V>,
{
    type Error = ValRef<'a>;
    fn try_from(v: ValRef<'a>) -> Result<VarRef<'a, V>, ValRef<'a>> {
        if TryInto::<&V>::try_into(v.as_norm()).is_ok() {
            Ok(VarRef {
                ptr: v.ptr,
                variant: std::marker::PhantomData,
            })
        } else {
            Err(v)
        }
    }
}

impl<P> Apply for ValRef<'_, P> {
    #[inline]
    fn do_apply<'a>(&self, args: &'a [ValId], inline: bool) -> Result<Application<'a>, Error> {
        self.ptr.do_apply(args, inline)
    }
}

impl<V: Value> From<VarRef<'_, V>> for ValueEnum {
    fn from(val: VarRef<V>) -> ValueEnum {
        val.as_enum().clone()
    }
}

impl<V: Value> From<VarRef<'_, V>> for NormalValue {
    fn from(val: VarRef<V>) -> NormalValue {
        val.as_norm().clone()
    }
}

impl<V: Value> From<VarRef<'_, V>> for ValId {
    fn from(val: VarRef<V>) -> ValId {
        val.clone_val()
    }
}

/// A private type which can only be constructed within the `value` crate: an implementation detail so that
/// `&ValId` cannot be `RefCast`ed to `&TypeId` outside the module (for type safety).
#[derive(Debug)]
pub struct Private {}

/// A wrapper over an `Arc<NormalValue>` with `ByAddress` semantics for `PartialEq`, `Eq` and `Hash`
/// Can only be constructed within the `value` crate: a user should never have direct access to these.
type NormAddr = PrivateByAddr<Arc<NormalValue>, Private>;

/// A wrapper over an `ArcBorrow<NormalValue>` with `ByAddress` semantics for `PartialEq`, `Eq` and `Hash`
/// Can only be constructed within the `value` crate: a user should never have direct access to these.
type NormRef<'a> = PrivateByAddr<ArcBorrow<'a, NormalValue>, Private>;

/// A normalized `rain` value
#[derive(Clone, Eq, PartialEq, Hash, RefCast)]
#[repr(transparent)]
pub struct NormalValue(pub(crate) PrivateValue);

impl NormalValue {
    /// Assert a given value is normalized
    pub(crate) fn assert_new(value: ValueEnum) -> NormalValue {
        NormalValue(PrivateValue(value))
    }
}

impl Deref for NormalValue {
    type Target = ValueEnum;
    fn deref(&self) -> &ValueEnum {
        &(self.0).0
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
        &(self.0).0
    }
}

impl From<NormalValue> for ValueEnum {
    #[inline]
    fn from(normal: NormalValue) -> ValueEnum {
        (normal.0).0
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
    fn do_apply<'a>(&self, args: &'a [ValId], inline: bool) -> Result<Application<'a>, Error> {
        self.deref().do_apply(args, inline)
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
    fn region(&self) -> RegionBorrow {
        self.deref().region()
    }
}

/// A wrapper around a `rain` value to assert refinement conditions.
/// Implementation detail: library consumers should not be able to construct this!
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PrivateValue(pub(crate) ValueEnum);

debug_from_display!(NormalValue);
pretty_display!(NormalValue, s, fmt => write!(fmt, "{}", s.deref()));

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
    /// Convert a value into a `NormalValue`
    fn into_norm(self) -> NormalValue;
    /// Convert a value into a `ValueEnum`
    fn into_enum(self) -> ValueEnum {
        self.into_norm().into()
    }
    /// Convert a value into a `ValId`
    fn into_val(self) -> ValId {
        self.into_norm().into()
    }
    /// Convert a value into a `TypeId`, if it is a type, otherwise return it
    fn try_into_ty(self) -> Result<TypeId, Self> {
        if self.is_ty() {
            Ok(self.into_val().coerce())
        } else {
            Err(self)
        }
    }
}

/// The dependencies of a value
#[derive(Debug, Copy, Clone, RefCast)]
#[repr(transparent)]
pub struct Deps<V>(pub V);

const DEP_SEARCH_STACK_SIZE: usize = 16;

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
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a ValId> + 'a {
        (0..self.len()).map(move |ix| self.0.get_dep(ix))
    }
    /// Collect the immediate dependencies of this value within a given depth range which match a given filter
    pub fn collect_deps<R, F>(&self, range: R, filter: F) -> Vec<ValId>
    where
        V: Clone,
        R: RangeBounds<usize>,
        F: Fn(&ValId) -> bool,
    {
        let mut result = Vec::new();
        // Simple edge case
        if range.contains(&self.0.depth()) {
            return vec![self.0.clone().into_val()];
        }
        let mut searched = FxHashSet::<&ValId>::default();
        let mut frontier: SmallVec<[&ValId; DEP_SEARCH_STACK_SIZE]> = self.iter().collect();
        while let Some(dep) = frontier.pop() {
            searched.insert(dep);
            if range.contains(&dep.depth()) {
                if filter(dep) {
                    result.push(dep.clone())
                }
            } else {
                frontier.extend(dep.deps().iter().filter(|dep| !searched.contains(dep)))
            }
        }
        result
    }
}

/// A wrapper for a reference
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, RefCast)]
#[repr(transparent)]
pub struct Borrowed<'a, V>(&'a V);

/// A depth-first search of a value's dependencies matching a given filter.
/// This filter maps the results, and may morph their dependencies and/or assert a certain value type.
/// Dependencies not matching the filter are ignored *along with all their descendants*.
/// May repeat dependencies.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DepDFS<V, F> {
    /// The frontier of this search
    frontier: Vec<(V, usize)>,
    /// The filter to apply
    filter: F,
}

impl<V, F> Iterator for DepDFS<V, F>
where
    V: Value,
    F: FnMut(&ValId) -> Option<V>,
{
    type Item = V;
    fn next(&mut self) -> Option<V> {
        loop {
            let mut push_to_top = None;
            {
                let (top, ix) = self.frontier.last_mut()?;
                while *ix < top.no_deps() {
                    *ix += 1;
                    if let Some(dep) = (self.filter)(top.get_dep(*ix - 1)) {
                        push_to_top = Some(dep);
                        break; // Push this to the top of the dependency stack, repeat
                    }
                }
            }
            if let Some(to_push) = push_to_top {
                self.frontier.push((to_push, 0));
                continue;
            } else {
                break;
            }
        }
        self.frontier.pop().map(|(b, _)| b)
    }
}

impl<'a, V, F> Iterator for DepDFS<Borrowed<'a, V>, F>
where
    V: Value,
    F: FnMut(&'a ValId) -> Option<&'a V>,
{
    type Item = &'a V;
    fn next(&mut self) -> Option<&'a V> {
        loop {
            let mut push_to_top = None;
            {
                let (top, ix) = self.frontier.last_mut()?;
                while *ix < top.0.no_deps() {
                    *ix += 1;
                    if let Some(dep) = (self.filter)(top.0.get_dep(*ix - 1)) {
                        push_to_top = Some(Borrowed(dep));
                        break; // Push this to the top of the dependency stack, repeat
                    }
                }
            }
            if let Some(to_push) = push_to_top {
                self.frontier.push((to_push, 0));
                continue;
            } else {
                break;
            }
        }
        self.frontier.pop().map(|(b, _)| b.0)
    }
}

/// A naive depth-first search of a value's dependencies matching a given filter.
/// A depth-first search of a value's dependencies matching a given filter.
/// This filter maps the results, and may morph their dependencies and/or assert a certain value type.
/// Dependencies not matching the filter are ignored *along with all their descendants*.
/// This search relies on the filter to mark nodes as already visited: if not, expect an explosion of memory use.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NaiveDFS<V, F> {
    /// The frontier of this search
    frontier: Vec<V>,
    /// The filter to apply
    filter: F,
    /// The value type
    value: std::marker::PhantomData<V>,
}

impl<V, F> Iterator for NaiveDFS<V, F>
where
    V: Value,
    F: FnMut(&ValId) -> Option<V>,
{
    type Item = V;
    fn next(&mut self) -> Option<V> {
        unimplemented!()
    }
}

/// A breadth-first search of a value's dependencies matching a given filter.
/// Dependencies not matching the filter are ignored *along with all their descendants*.
/// May repeat dependencies.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DepBFS<V, F> {
    /// The frontier of this search
    frontier: Vec<V>,
    /// The filter to apply
    filter: F,
    /// The value type
    value: std::marker::PhantomData<V>,
}

impl<V, F> Iterator for DepBFS<V, F>
where
    V: Value,
    F: FnMut(&ValId) -> Option<V>,
{
    type Item = V;
    fn next(&mut self) -> Option<V> {
        unimplemented!()
    }
}

impl<V: Value> std::ops::Index<usize> for Deps<V> {
    type Output = ValId;
    fn index(&self, ix: usize) -> &ValId {
        self.0.get_dep(ix)
    }
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

impl Apply for ValueEnum {
    #[inline]
    fn do_apply<'a>(&self, args: &'a [ValId], inline: bool) -> Result<Application<'a>, Error> {
        forv! {match (self) {
            v => v.do_apply(args, inline),
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

enum_convert! {
    // ValueEnum injection:
    impl Injection<ValueEnum> for Sexpr {
        match
            other if *other == () => Ok(Sexpr::unit()),
            other => Ok(Sexpr::singleton(ValId::from(other))),
    }
    impl TryFromRef<ValueEnum> for Sexpr {}
    impl InjectionRef<ValueEnum> for Parameter {}
    impl Injection<ValueEnum> for Tuple {
        match
            other if *other == () => Ok(Tuple::unit()),
    }
    impl TryFromRef<ValueEnum> for Tuple {}
    impl Injection<ValueEnum> for Product {
        match
            other if *other == Unit => Ok(Product::unit_ty()),
    }
    impl TryFromRef<ValueEnum> for Product {}
    impl InjectionRef<ValueEnum> for Universe {}
    impl InjectionRef<ValueEnum> for Finite {}
    impl InjectionRef<ValueEnum> for Index {}
    impl InjectionRef<ValueEnum> for Pi {}
    impl InjectionRef<ValueEnum> for Lambda {}
    impl InjectionRef<ValueEnum> for Gamma {}
    impl InjectionRef<ValueEnum> for Phi {}
    impl InjectionRef<ValueEnum> for Logical {}

    // NormalValue injection.
    impl TryFrom<NormalValue> for Sexpr {
        as ValueEnum,
        match
            other if *other == () => Ok(Sexpr::unit()),
            other => Ok(Sexpr::singleton(ValId::from(other))),
    }
    impl TryFromRef<NormalValue> for Sexpr { as ValueEnum, }
    impl TryFrom<NormalValue> for Parameter { as ValueEnum, }
    impl TryFromRef<NormalValue> for Parameter { as ValueEnum, }
    impl TryFrom<NormalValue> for Tuple {
        as ValueEnum,
        match
            other if *other == () => Ok(Tuple::unit()),
    }
    impl TryFromRef<NormalValue> for Tuple { as ValueEnum, }
    impl TryFrom<NormalValue> for Product { as ValueEnum, }
    impl TryFromRef<NormalValue> for Product { as ValueEnum, }
    impl TryFrom<NormalValue> for Universe { as ValueEnum, }
    impl TryFromRef<NormalValue> for Universe { as ValueEnum, }
    impl TryFrom<NormalValue> for Finite { as ValueEnum, }
    impl TryFromRef<NormalValue> for Finite { as ValueEnum, }
    impl TryFrom<NormalValue> for Index { as ValueEnum, }
    impl TryFromRef<NormalValue> for Index { as ValueEnum, }
    impl TryFrom<NormalValue> for Pi { as ValueEnum, }
    impl TryFromRef<NormalValue> for Pi { as ValueEnum, }
    impl TryFrom<NormalValue> for Lambda { as ValueEnum, }
    impl TryFromRef<NormalValue> for Lambda { as ValueEnum, }
    impl TryFrom<NormalValue> for Gamma { as ValueEnum, }
    impl TryFromRef<NormalValue> for Gamma { as ValueEnum, }
    impl TryFrom<NormalValue> for Phi { as ValueEnum, }
    impl TryFromRef<NormalValue> for Phi { as ValueEnum, }
    impl TryFrom<NormalValue> for Logical { as ValueEnum, }
    impl TryFromRef<NormalValue> for Logical { as ValueEnum, }
}

impl From<Sexpr> for NormalValue {
    fn from(sexpr: Sexpr) -> NormalValue {
        if sexpr == () {
            return ().into();
        }
        if sexpr.len() == 1 {
            return sexpr[0].as_norm().clone();
        }
        NormalValue::assert_new(ValueEnum::Sexpr(sexpr))
    }
}

impl From<Parameter> for NormalValue {
    fn from(param: Parameter) -> NormalValue {
        NormalValue::assert_new(ValueEnum::Parameter(param))
    }
}

impl From<Tuple> for NormalValue {
    fn from(tuple: Tuple) -> NormalValue {
        if tuple == () {
            return ().into();
        }
        NormalValue::assert_new(ValueEnum::Tuple(tuple))
    }
}

impl From<Product> for NormalValue {
    fn from(product: Product) -> NormalValue {
        if product == Unit {
            return Unit.into();
        }
        NormalValue::assert_new(ValueEnum::Product(product))
    }
}

impl From<Universe> for NormalValue {
    fn from(universe: Universe) -> NormalValue {
        NormalValue::assert_new(ValueEnum::Universe(universe))
    }
}

impl From<Bool> for ValueEnum {
    fn from(b: Bool) -> ValueEnum {
        ValueEnum::BoolTy(b)
    }
}

impl TryFrom<ValueEnum> for Bool {
    type Error = ValueEnum;
    fn try_from(val: ValueEnum) -> Result<Bool, ValueEnum> {
        match val {
            ValueEnum::BoolTy(b) => Ok(b),
            v => Err(v),
        }
    }
}

impl<'a> TryFrom<&'a ValueEnum> for &'a Bool {
    type Error = &'a ValueEnum;
    fn try_from(val: &'a ValueEnum) -> Result<&'a Bool, &'a ValueEnum> {
        match val {
            ValueEnum::BoolTy(b) => Ok(b),
            v => Err(v),
        }
    }
}

impl From<Bool> for NormalValue {
    fn from(b: Bool) -> NormalValue {
        NormalValue::assert_new(ValueEnum::BoolTy(b))
    }
}

impl TryFrom<NormalValue> for Bool {
    type Error = NormalValue;
    fn try_from(val: NormalValue) -> Result<Bool, NormalValue> {
        match val.deref() {
            ValueEnum::BoolTy(b) => Ok(*b),
            _ => Err(val),
        }
    }
}

impl<'a> TryFrom<&'a NormalValue> for &'a Bool {
    type Error = &'a NormalValue;
    fn try_from(val: &'a NormalValue) -> Result<&'a Bool, &'a NormalValue> {
        match val.deref() {
            ValueEnum::BoolTy(b) => Ok(b),
            _ => Err(val),
        }
    }
}

impl From<bool> for ValueEnum {
    fn from(b: bool) -> ValueEnum {
        ValueEnum::Bool(b)
    }
}

impl TryFrom<ValueEnum> for bool {
    type Error = ValueEnum;
    fn try_from(val: ValueEnum) -> Result<bool, ValueEnum> {
        match val {
            ValueEnum::Bool(b) => Ok(b),
            v => Err(v),
        }
    }
}

impl<'a> TryFrom<&'a ValueEnum> for &'a bool {
    type Error = &'a ValueEnum;
    fn try_from(val: &'a ValueEnum) -> Result<&'a bool, &'a ValueEnum> {
        match val {
            ValueEnum::Bool(b) => Ok(b),
            v => Err(v),
        }
    }
}

impl From<bool> for NormalValue {
    fn from(b: bool) -> NormalValue {
        NormalValue::assert_new(ValueEnum::Bool(b))
    }
}

impl TryFrom<NormalValue> for bool {
    type Error = NormalValue;
    fn try_from(val: NormalValue) -> Result<bool, NormalValue> {
        match val.deref() {
            ValueEnum::Bool(b) => Ok(*b),
            _ => Err(val),
        }
    }
}

impl<'a> TryFrom<&'a NormalValue> for &'a bool {
    type Error = &'a NormalValue;
    fn try_from(val: &'a NormalValue) -> Result<&'a bool, &'a NormalValue> {
        match val.deref() {
            ValueEnum::Bool(b) => Ok(b),
            _ => Err(val),
        }
    }
}

impl From<Finite> for NormalValue {
    fn from(finite: Finite) -> NormalValue {
        NormalValue::assert_new(ValueEnum::Finite(finite))
    }
}

impl From<Index> for NormalValue {
    fn from(ix: Index) -> NormalValue {
        NormalValue::assert_new(ValueEnum::Index(ix))
    }
}

impl From<Pi> for NormalValue {
    fn from(p: Pi) -> NormalValue {
        NormalValue::assert_new(ValueEnum::Pi(p))
    }
}

impl From<Lambda> for NormalValue {
    fn from(l: Lambda) -> NormalValue {
        NormalValue::assert_new(ValueEnum::Lambda(l))
    }
}

impl From<Gamma> for NormalValue {
    fn from(g: Gamma) -> NormalValue {
        NormalValue::assert_new(ValueEnum::Gamma(g))
    }
}

impl From<Phi> for NormalValue {
    fn from(p: Phi) -> NormalValue {
        NormalValue::assert_new(ValueEnum::Phi(p))
    }
}

impl From<Logical> for NormalValue {
    fn from(l: Logical) -> NormalValue {
        NormalValue::assert_new(ValueEnum::Logical(l))
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
    match (v) { v => write!(fmt, "{}", v) }
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
    fn region(&self) -> RegionBorrow {
        forv!(match (self) {
            s => s.region(),
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

    impl<P> PrettyPrint for ValId<P> {
        #[inline]
        fn prettyprint<I: From<usize> + Display>(
            &self,
            printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            self.as_norm().prettyprint(printer, fmt)
        }
    }

    impl<P> PrettyPrint for ValRef<'_, P> {
        #[inline]
        fn prettyprint<I: From<usize> + Display>(
            &self,
            printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            self.as_norm().prettyprint(printer, fmt)
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
