/*!
`rain` values
*/
use crate::util::{hash_cache::Cache, PrivateByAddr};
use crate::{debug_from_display, enum_convert, forv, pretty_display};
use lazy_static::lazy_static;
use ref_cast::RefCast;
use std::borrow::Borrow;
use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::Hash;
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
use typing::{Type, Typed};
use universe::Universe;

lazy_static! {
    /// The global `rain` value cache
    pub static ref VALUE_CACHE: Cache<NormalValue> = Cache::new();
}

/// A reference-counted, hash-consed `rain` value
#[derive(Clone, Eq, PartialEq, Hash, RefCast)]
#[repr(transparent)]
pub struct ValId(NormAddr);

impl ValId {
    /// Directly construct a `ValId` from a `NormalValue`, deduplicating but not performing any other transformation/caching.
    /// Useful to prevent infinite regress in e.g. cached constructors for `()`
    #[inline]
    pub fn direct_new<V: Into<NormalValue>>(v: V) -> ValId {
        let norm = v.into();
        ValId(NormAddr::make(VALUE_CACHE.cache(norm), Private {}))
    }
    /// Deduplicate an `Arc<NormalValue>` to yield a `ValId`
    #[inline]
    pub fn dedup(norm: Arc<NormalValue>) -> ValId {
        ValId(NormAddr::make(VALUE_CACHE.cache(norm), Private {}))
    }
    /// Borrow this value
    #[inline]
    pub fn borrow_val(&self) -> ValRef {
        ValRef(self.0.borrow_arc())
    }
    /// Get this `ValId` as a `ValueEnum`
    #[inline]
    pub fn as_enum(&self) -> &ValueEnum {
        &self.0
    }
    /// Get this `ValId` as a `NormalValue`
    #[inline]
    pub fn as_norm(&self) -> &NormalValue {
        &self.0
    }
}

impl<'a, V> PartialEq<VarRef<'a, V>> for ValId {
    fn eq(&self, other: &VarRef<'a, V>) -> bool {
        self.0 == other.ptr
    }
}

impl<V> PartialEq<VarId<V>> for ValId {
    fn eq(&self, other: &VarId<V>) -> bool {
        self.0 == other.ptr
    }
}

impl PartialEq<TypeId> for ValId {
    fn eq(&self, other: &TypeId) -> bool {
        self.0 == other.0
    }
}

impl<'a> PartialEq<ValRef<'a>> for ValId {
    fn eq(&self, other: &ValRef<'a>) -> bool {
        self.0 == other.0
    }
}

impl<'a> PartialEq<TypeRef<'a>> for ValId {
    fn eq(&self, other: &TypeRef<'a>) -> bool {
        self.0 == other.0
    }
}

impl Deref for ValId {
    type Target = NormalValue;
    #[inline]
    fn deref(&self) -> &NormalValue {
        &self.0.addr
    }
}

impl From<NormalValue> for ValId {
    #[inline]
    fn from(value: NormalValue) -> ValId {
        ValId(NormAddr::make(VALUE_CACHE.cache(value), Private {}))
    }
}

impl From<Arc<NormalValue>> for ValId {
    #[inline]
    fn from(value: Arc<NormalValue>) -> ValId {
        ValId(NormAddr::make(VALUE_CACHE.cache(value), Private {}))
    }
}

impl Live for ValId {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.deref().lifetime()
    }
}

impl Typed for ValId {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.deref().ty()
    }
}

impl Value for ValId {
    #[inline]
    fn no_deps(&self) -> usize {
        self.deref().no_deps()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        self.deref().get_dep(ix)
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

/// A reference to a `rain` value
#[derive(Copy, Clone, Eq, PartialEq, Hash, RefCast)]
#[repr(transparent)]
pub struct ValRef<'a>(NormRef<'a>);

impl<'a> ValRef<'a> {
    /// Clone this value reference as a `ValId`
    #[inline]
    pub fn clone_val(&self) -> ValId {
        ValId(self.0.clone_arc())
    }
    /// Get this `ValRef` as a `ValueEnum`
    #[inline]
    pub fn as_enum(&self) -> &'a ValueEnum {
        self.0.get()
    }
    /// Get this `TypeRef` as a `NormalValue`
    #[inline]
    pub fn as_norm(&self) -> &'a NormalValue {
        self.0.get()
    }
}

impl<'a, V> PartialEq<VarRef<'a, V>> for ValRef<'a> {
    fn eq(&self, other: &VarRef<'a, V>) -> bool {
        self.0 == other.ptr
    }
}

impl<'a, V> PartialEq<VarId<V>> for ValRef<'a> {
    fn eq(&self, other: &VarId<V>) -> bool {
        self.0 == other.ptr
    }
}

impl<'a> PartialEq<ValId> for ValRef<'a> {
    fn eq(&self, other: &ValId) -> bool {
        self.0 == other.0
    }
}

impl<'a> PartialEq<TypeId> for ValRef<'a> {
    fn eq(&self, other: &TypeId) -> bool {
        self.0 == other.0
    }
}

impl<'a> PartialEq<TypeRef<'a>> for ValRef<'a> {
    fn eq(&self, other: &TypeRef<'a>) -> bool {
        self.0 == other.0
    }
}

impl<'a> Deref for ValRef<'a> {
    type Target = NormalValue;
    #[inline]
    fn deref(&self) -> &NormalValue {
        &self.0.addr
    }
}

impl Live for ValRef<'_> {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.deref().lifetime()
    }
}

impl Typed for ValRef<'_> {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.deref().ty()
    }
}

impl Value for ValRef<'_> {
    #[inline]
    fn no_deps(&self) -> usize {
        self.deref().no_deps()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        self.deref().get_dep(ix)
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

debug_from_display!(ValId);
pretty_display!(ValId, s, fmt  => write!(fmt, "{}", s.deref()));
debug_from_display!(ValRef<'_>);
pretty_display!(ValRef<'_>, s, fmt  => write!(fmt, "{}", s.deref()));

//TODO: consider making `TypeId` into `VarId<TypeEnum>`

/// A reference-counted, hash-consed `rain` type
#[derive(Clone, Eq, PartialEq, Hash, RefCast)]
#[repr(transparent)]
pub struct TypeId(NormAddr);

impl<'a, V> PartialEq<VarRef<'a, V>> for TypeId {
    fn eq(&self, other: &VarRef<'a, V>) -> bool {
        self.0 == other.ptr
    }
}

impl<V> PartialEq<VarId<V>> for TypeId {
    fn eq(&self, other: &VarId<V>) -> bool {
        self.0 == other.ptr
    }
}

impl PartialEq<ValId> for TypeId {
    fn eq(&self, other: &ValId) -> bool {
        self.0 == other.0
    }
}

impl<'a> PartialEq<ValRef<'a>> for TypeId {
    fn eq(&self, other: &ValRef<'a>) -> bool {
        self.0 == other.0
    }
}

impl<'a> PartialEq<TypeRef<'a>> for TypeId {
    fn eq(&self, other: &TypeRef<'a>) -> bool {
        self.0 == other.0
    }
}

impl Deref for TypeId {
    type Target = ValId;
    #[inline]
    fn deref(&self) -> &ValId {
        RefCast::ref_cast(&self.0)
    }
}

impl TypeId {
    /// Directly construct a `ValId` from a `NormalValue`, deduplicating but not performing any other transformation/caching.
    /// Useful to prevent infinite regress in e.g. cached constructors for `()`
    pub fn direct_new<V>(v: V) -> TypeId
    where
        V: Type + Into<NormalValue>,
    {
        let norm: NormalValue = v.into();
        TypeId(NormAddr::make(VALUE_CACHE.cache(norm), Private {}))
    }
    /// Get this `TypeId` as a `ValId`
    #[inline]
    pub fn as_val(&self) -> &ValId {
        &self
    }
    /// Borrow a `TypeId`
    #[inline]
    pub fn borrow_ty(&self) -> TypeRef {
        TypeRef(self.0.borrow_arc())
    }
    /// Get this `TypeId` as a `ValueEnum`
    #[inline]
    pub fn as_enum(&self) -> &ValueEnum {
        &self.0
    }
    /// Get this `TypeRef` as a `NormalValue`
    #[inline]
    pub fn as_norm(&self) -> &NormalValue {
        &self.0
    }
}

impl From<TypeId> for ValId {
    #[inline]
    fn from(ty: TypeId) -> ValId {
        ValId(ty.0)
    }
}

impl Live for TypeId {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.deref().lifetime()
    }
}

impl Typed for TypeId {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.deref().ty()
    }
}

impl Value for TypeId {
    #[inline]
    fn no_deps(&self) -> usize {
        self.deref().no_deps()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        self.deref().get_dep(ix)
    }
}

impl From<TypeId> for ValueEnum {
    fn from(val: TypeId) -> ValueEnum {
        val.as_enum().clone()
    }
}

impl From<TypeId> for NormalValue {
    fn from(val: TypeId) -> NormalValue {
        val.as_norm().clone()
    }
}

impl Type for TypeId {
    #[inline]
    fn universe(&self) -> UniverseRef {
        match self.as_enum() {
            ValueEnum::Universe(u) => u.universe(),
            ValueEnum::Product(p) => p.universe(),
            ValueEnum::Parameter(_p) => unimplemented!(),
            _ => panic!("Impossible!"),
        }
    }
}

/// A reference to a `rain` type
#[derive(Copy, Clone, Eq, PartialEq, Hash, RefCast)]
#[repr(transparent)]
pub struct TypeRef<'a>(NormRef<'a>);

impl<'a> TypeRef<'a> {
    /// Clone this type reference as a `TypeRef`
    #[inline]
    pub fn clone_ty(&self) -> TypeId {
        TypeId(self.0.clone_arc())
    }
    /// Get this `TypeRef` as `ValRef`
    #[inline]
    pub fn as_val(&self) -> ValRef<'a> {
        ValRef(self.0)
    }
    /// Get this `TypeRef` as a `ValueEnum`
    #[inline]
    pub fn as_enum(&self) -> &'a ValueEnum {
        self.0.get()
    }
    /// Get this `TypeRef` as a `NormalValue`
    #[inline]
    pub fn as_norm(&self) -> &'a NormalValue {
        self.0.get()
    }
}

impl<'a, V> PartialEq<VarRef<'a, V>> for TypeRef<'a> {
    fn eq(&self, other: &VarRef<'a, V>) -> bool {
        self.0 == other.ptr
    }
}

impl<'a, V> PartialEq<VarId<V>> for TypeRef<'a> {
    fn eq(&self, other: &VarId<V>) -> bool {
        self.0 == other.ptr
    }
}

impl<'a> PartialEq<ValId> for TypeRef<'a> {
    fn eq(&self, other: &ValId) -> bool {
        self.0 == other.0
    }
}

impl<'a> PartialEq<TypeId> for TypeRef<'a> {
    fn eq(&self, other: &TypeId) -> bool {
        self.0 == other.0
    }
}

impl<'a> PartialEq<ValRef<'a>> for TypeRef<'a> {
    fn eq(&self, other: &ValRef<'a>) -> bool {
        self.0 == other.0
    }
}

impl<'a> Deref for TypeRef<'a> {
    type Target = ValRef<'a>;
    #[inline]
    fn deref(&self) -> &ValRef<'a> {
        RefCast::ref_cast(&self.0)
    }
}

impl<'a> From<TypeRef<'a>> for ValRef<'a> {
    #[inline]
    fn from(t: TypeRef<'a>) -> ValRef<'a> {
        t.as_val()
    }
}

impl Live for TypeRef<'_> {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.deref().lifetime()
    }
}

impl Typed for TypeRef<'_> {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.deref().ty()
    }
}

impl Value for TypeRef<'_> {
    #[inline]
    fn no_deps(&self) -> usize {
        self.deref().no_deps()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        self.deref().get_dep(ix)
    }
}

impl From<TypeRef<'_>> for ValueEnum {
    fn from(val: TypeRef) -> ValueEnum {
        val.as_enum().clone()
    }
}

impl From<TypeRef<'_>> for NormalValue {
    fn from(val: TypeRef) -> NormalValue {
        val.as_norm().clone()
    }
}

debug_from_display!(TypeId);
pretty_display!(TypeId, s, fmt => write!(fmt, "{}", s.deref()));
debug_from_display!(TypeRef<'_>);
pretty_display!(TypeRef<'_>, s, fmt => write!(fmt, "{}", s.deref()));

/// A value guaranteed to be a certain `ValueEnum` variant (may not be an actual variant)
#[derive(Eq, Hash, RefCast)]
#[repr(transparent)]
pub struct VarId<Variant> {
    ptr: NormAddr,
    variant: std::marker::PhantomData<Variant>,
}

impl<'a, U, V> PartialEq<VarRef<'a, U>> for VarId<V> {
    fn eq(&self, other: &VarRef<'a, U>) -> bool {
        self.ptr == other.ptr
    }
}

impl<U, V> PartialEq<VarId<U>> for VarId<V> {
    fn eq(&self, other: &VarId<U>) -> bool {
        self.ptr == other.ptr
    }
}

impl<V> PartialEq<ValId> for VarId<V> {
    fn eq(&self, other: &ValId) -> bool {
        self.ptr == other.0
    }
}

impl<'a, V> PartialEq<ValRef<'a>> for VarId<V> {
    fn eq(&self, other: &ValRef<'a>) -> bool {
        self.ptr == other.0
    }
}

impl<V> PartialEq<TypeId> for VarId<V> {
    fn eq(&self, other: &TypeId) -> bool {
        self.ptr == other.0
    }
}

impl<'a, V> PartialEq<TypeRef<'a>> for VarId<V> {
    fn eq(&self, other: &TypeRef<'a>) -> bool {
        self.ptr == other.0
    }
}

impl<'a, V> Clone for VarId<V> {
    #[inline]
    fn clone(&self) -> VarId<V> {
        VarId {
            ptr: self.ptr.clone(),
            variant: self.variant,
        }
    }
}

impl<'a, V> VarId<V> {
    /// Directly construct a `ValId` from a `V`, deduplicating but not performing any other transformation/caching.
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
    /// Get this `VarId` as a `NormalValue`
    pub fn as_norm(&self) -> &NormalValue {
        self.ptr.deref()
    }
    /// Get this `VarId` as a `ValueEnum`
    pub fn as_enum(&self) -> &ValueEnum {
        self.ptr.deref()
    }
    /// Get this `VarId` as a `ValId`
    pub fn as_val(&self) -> &ValId {
        RefCast::ref_cast(&self.ptr)
    }
    /// Get this `VarId` as a `TypeId`
    pub fn as_ty(&self) -> &TypeId
    where
        V: Type,
    {
        RefCast::ref_cast(&self.ptr)
    }
    /// Borrow this `VarId` as a `VarRef`
    pub fn borrow_var(&self) -> VarRef<V> {
        VarRef {
            ptr: self.ptr.borrow_arc(),
            variant: self.variant,
        }
    }
    /// Borrow this `VarId` as a `ValRef`
    pub fn borrow_val(&self) -> ValRef {
        ValRef(self.ptr.borrow_arc())
    }
    /// Borrow this `VarId` as a `TypeRef`
    pub fn borrow_ty(&self) -> TypeRef
    where
        V: Type,
    {
        TypeRef(self.ptr.borrow_arc())
    }
}

impl<V> From<VarId<V>> for ValId {
    fn from(v: VarId<V>) -> ValId {
        ValId(v.ptr)
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
            _ => panic!("Impossible!"),
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
                ptr: v.0,
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
            ptr: valid.0,
            variant: std::marker::PhantomData,
        }
    }
}

impl<V> From<VarId<V>> for TypeId
where
    V: Type,
{
    fn from(v: VarId<V>) -> TypeId {
        TypeId(v.ptr)
    }
}

impl<'a, V> From<VarRef<'a, V>> for TypeRef<'a>
where
    V: Type,
{
    fn from(v: VarRef<'a, V>) -> TypeRef<'a> {
        TypeRef(v.ptr)
    }
}

impl<V: Typed> Typed for VarId<V> {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.as_norm().ty()
    }
}

impl<V: Value> Value for VarId<V> {
    #[inline]
    fn no_deps(&self) -> usize {
        self.deref().no_deps()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        self.as_norm().get_dep(ix)
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

/// A reference-counted pointer to a value guaranteed to be a typing universe
pub type UniverseId = VarId<Universe>;

/// A pointer to a value guaranteed to be a typing universe
pub type UniverseRef<'a> = VarRef<'a, Universe>;

/// A (*normalized, consed*) borrowed value guaranteed to be a certain `ValueEnum` variant (may not be an actual variant, e.g. `()` or `Unit`)
#[derive(Eq, Hash, RefCast)]
#[repr(transparent)]
pub struct VarRef<'a, Variant> {
    ptr: NormRef<'a>,
    variant: std::marker::PhantomData<Variant>,
}

impl<'a, U, V> PartialEq<VarRef<'a, U>> for VarRef<'a, V> {
    fn eq(&self, other: &VarRef<'a, U>) -> bool {
        self.ptr == other.ptr
    }
}

impl<'a, U, V> PartialEq<VarId<U>> for VarRef<'a, V> {
    fn eq(&self, other: &VarId<U>) -> bool {
        self.ptr == other.ptr
    }
}

impl<'a, V> PartialEq<ValId> for VarRef<'a, V> {
    fn eq(&self, other: &ValId) -> bool {
        self.ptr == other.0
    }
}

impl<'a, V> PartialEq<ValRef<'a>> for VarRef<'a, V> {
    fn eq(&self, other: &ValRef<'a>) -> bool {
        self.ptr == other.0
    }
}

impl<'a, V> PartialEq<TypeId> for VarRef<'a, V> {
    fn eq(&self, other: &TypeId) -> bool {
        self.ptr == other.0
    }
}

impl<'a, V> PartialEq<TypeRef<'a>> for VarRef<'a, V> {
    fn eq(&self, other: &TypeRef<'a>) -> bool {
        self.ptr == other.0
    }
}

impl<'a, V> Clone for VarRef<'a, V> {
    #[inline]
    fn clone(&self) -> VarRef<'a, V> {
        VarRef {
            ptr: self.ptr,
            variant: self.variant,
        }
    }
}

impl<'a, V> Copy for VarRef<'a, V> {}

impl<'a, V> VarRef<'a, V> {
    /// Get this `VarRef` as a `NormalValue`
    pub fn as_norm(&self) -> &'a NormalValue {
        self.ptr.get()
    }
    /// Get this `VarRef` as a `ValueEnum`
    pub fn as_enum(&self) -> &'a ValueEnum {
        self.ptr.get()
    }
    /// Get this `VarRef` as a `ValRef`
    pub fn as_val(&self) -> ValRef<'a> {
        ValRef(self.ptr)
    }
    /// Get this `VarRef` as a `TypeRef`
    pub fn as_ty(&self) -> TypeRef<'a>
    where
        V: Type,
    {
        TypeRef(self.ptr)
    }
    /// Clone this `VarRef` as a `ValId`
    pub fn clone_val(&self) -> ValId {
        self.as_val().clone_val()
    }
    /// Clone this `VarRef` as a `VarId`
    pub fn clone_var(&self) -> VarId<V> {
        VarId {
            ptr: self.ptr.clone_arc(),
            variant: self.variant,
        }
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
            _ => panic!("Impossible!"),
        }
    }
}

impl<V> Debug for VarId<V> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(self.ptr.deref(), fmt)
    }
}

impl<V> Debug for VarRef<'_, V> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(self.ptr.get(), fmt)
    }
}

impl<V> Display for VarId<V> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        Display::fmt(self.ptr.deref(), fmt)
    }
}

impl<V> Display for VarRef<'_, V> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        Display::fmt(self.ptr.get(), fmt)
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
                ptr: v.0,
                variant: std::marker::PhantomData,
            })
        } else {
            Err(v)
        }
    }
}

impl<V: Typed> Typed for VarRef<'_, V> {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.as_norm().ty()
    }
}

impl<'a, V: Value> Value for VarRef<'a, V> {
    #[inline]
    fn no_deps(&self) -> usize {
        self.deref().no_deps()
    }
    #[inline]
    fn get_dep<'b>(&'b self, ix: usize) -> &'b ValId {
        self.as_norm().get_dep(ix)
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

impl Typed for NormalValue {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.0.ty()
    }
}

impl Value for NormalValue {
    #[inline]
    fn no_deps(&self) -> usize {
        self.0.no_deps()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        self.0.get_dep(ix)
    }
}

debug_from_display!(NormalValue);
pretty_display!(NormalValue, s, fmt => write!(fmt, "{}", s.deref()));

/// A trait implemented by `rain` values
pub trait Value: Into<NormalValue> + Into<ValueEnum> + Typed {
    /// Get the number of dependencies of this value
    fn no_deps(&self) -> usize;
    /// Get a given dependency of this value
    fn get_dep(&self, dep: usize) -> &ValId;
    /// Get the dependencies of this value
    #[inline]
    fn get_deps(&self) -> &Deps<Self> {
        RefCast::ref_cast(self)
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
    /// Iterate over the dependencies of this value
    pub fn iter<'a>(&'a self) -> impl Iterator<Item=&'a ValId> + 'a {
        (0..self.len()).map(move |ix| self.0.get_dep(ix))
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
}

impl From<Sexpr> for NormalValue {
    fn from(sexpr: Sexpr) -> NormalValue {
        if sexpr == () {
            return ().into();
        }
        if sexpr.len() == 1 {
            return sexpr[0].as_norm().clone();
        }
        NormalValue(ValueEnum::Sexpr(sexpr))
    }
}

impl From<Parameter> for NormalValue {
    fn from(param: Parameter) -> NormalValue {
        NormalValue(ValueEnum::Parameter(param))
    }
}

impl From<Tuple> for NormalValue {
    fn from(tuple: Tuple) -> NormalValue {
        if tuple == () {
            return ().into();
        }
        NormalValue(ValueEnum::Tuple(tuple))
    }
}

impl From<Product> for NormalValue {
    fn from(product: Product) -> NormalValue {
        if product == Unit {
            return Unit.into();
        }
        NormalValue(ValueEnum::Product(product))
    }
}

impl From<Universe> for NormalValue {
    fn from(universe: Universe) -> NormalValue {
        NormalValue(ValueEnum::Universe(universe))
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

impl Typed for ValueEnum {
    fn ty(&self) -> TypeRef {
        forv!(match (self) {
            s => s.ty(),
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
                crate::value::TypeId(crate::value::ValId::from(v).0)
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
            printer: &mut PrettyPrinter,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            //TODO: this
            self.deref().prettyprint(printer, fmt)
        }
    }

    impl PrettyPrint for ValRef<'_> {
        #[inline]
        fn prettyprint(
            &self,
            printer: &mut PrettyPrinter,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            //TODO: this
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
