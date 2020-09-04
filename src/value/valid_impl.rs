use super::*;
use std::hash::Hasher;
use std::marker::PhantomData;

// Statics

lazy_static! {
    /// The global `rain` value cache
    pub static ref VALUE_CACHE: DashCache<Arc<NormalValue>> = DashCache::new();
}

// Garbage collection
impl<P> Drop for ValId<P> {
    fn drop(&mut self) {
        VALUE_CACHE.try_gc(&mut self.ptr);
    }
}

// Equality

impl<P> Eq for ValId<P> {}

impl<P> Eq for ValRef<'_, P> {}

impl<'a, P, Q> PartialEq<ValRef<'a, P>> for ValId<Q> {
    fn eq(&self, other: &ValRef<'a, P>) -> bool {
        std::ptr::eq(self.as_val().deref(), other.as_val().deref())
    }
}

impl<P, Q> PartialEq<ValId<P>> for ValId<Q> {
    fn eq(&self, other: &ValId<P>) -> bool {
        std::ptr::eq(self.as_val().deref(), other.as_val().deref())
    }
}

impl<'a, U, V> PartialEq<ValRef<'a, U>> for ValRef<'a, V> {
    fn eq(&self, other: &ValRef<'a, U>) -> bool {
        std::ptr::eq(self.as_val().deref(), other.as_val().deref())
    }
}

impl<'a, U, V> PartialEq<ValId<U>> for ValRef<'a, V> {
    fn eq(&self, other: &ValId<U>) -> bool {
        std::ptr::eq(self.as_val().deref(), other.as_val().deref())
    }
}

// Copy and clone

impl<P> Clone for ValId<P> {
    #[inline]
    fn clone(&self) -> ValId<P> {
        ValId {
            ptr: self.ptr.clone(),
            variant: self.variant,
        }
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

// Hash

impl<P> Hash for ValId<P> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.as_ptr(), hasher)
    }
}

impl<'a, P> Hash for ValRef<'a, P> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.as_ptr(), hasher)
    }
}

// Base methods (implemented on unqualified `ValId` pointers)

impl ValId {
    /// Directly construct a `ValId` from a `NormalValue`, deduplicating but not performing any other transformation/caching.
    /// Useful to prevent infinite regress in e.g. cached constructors for `()`
    #[inline]
    pub fn direct_new<V: Into<NormalValue>>(v: V) -> ValId {
        let norm = v.into();
        ValId {
            ptr: VALUE_CACHE.cache(norm),
            variant: PhantomData,
        }
    }
    /// Deduplicate an `Arc<NormalValue>` to yield a `ValId`
    #[inline]
    pub fn dedup(norm: Arc<NormalValue>) -> ValId {
        ValId {
            ptr: VALUE_CACHE.cache(norm),
            variant: PhantomData,
        }
    }
    /// Perform a substitution. Here to avoid code duplication during monomorphization
    pub fn substitute_impl(&self, ctx: &mut EvalCtx) -> Result<ValId, Error> {
        if let Some(value) = ctx.try_evaluate(self) {
            return Ok(value);
        }
        let result: ValId = self.deref().substitute(ctx)?;
        ctx.substitute_unchecked(self.clone(), result.clone())?;
        Ok(result)
    }
}

// General methods (implemented on all `ValId` pointers)

impl<P> ValId<P> {
    /// Get this `ValId<P>` as a `NormalValue`
    #[inline]
    pub fn as_norm(&self) -> &NormalValue {
        self.ptr.deref()
    }
    /// Get this `ValId<P>` as a `ValueEnum`
    #[inline]
    pub fn as_enum(&self) -> &ValueEnum {
        self.ptr.deref()
    }
    /// Get this `ValId<P>` as a `ValId`
    #[inline]
    pub fn as_val(&self) -> &ValId {
        self.coerce_ref()
    }
    /// Get this `ValId<P>` as a `NormalValue<P>`
    #[inline]
    pub fn as_pred(&self) -> &NormalValue<P> {
        self.ptr.deref().coerce_ref()
    }
    /// Get the pointer behind this `ValId`
    #[inline]
    pub fn as_ptr(&self) -> *const NormalValue {
        self.as_norm() as *const NormalValue
    }
    /// Coerce a pointer into a `ValId`
    ///
    /// # Safety
    /// This function can only be called with the result of `into_ptr` for `ValId`
    #[inline]
    pub unsafe fn from_raw(ptr: *const NormalValue) -> ValId<P> {
        ValId {
            ptr: Arc::from_raw(ptr),
            variant: PhantomData,
        }
    }
    /// Get the `Arc` underlying this `ValId<P>`, if any
    ///
    /// # Implementation notes
    /// Currently, a `ValId` is always behind an `Arc`, but we might use stowaway/union techniques later to improve performance.
    /// In this case, it makes sense. Furthermore, if this is *not* converted to a `ValId` or explicitly dropped within the
    /// `VALUE_CACHE`, there may be a resource leak until the same value is destroyed again (which may require it being created
    /// again if this was the last reference outside the `VALUE_CACHE`).
    #[inline]
    pub fn into_arc(self) -> Option<Arc<NormalValue>> {
        Some(unsafe { std::mem::transmute(self) })
    }
    /// Get the address behind this `ValId`
    #[inline]
    pub fn as_addr(&self) -> ValAddr {
        ValAddr(self.as_norm() as *const NormalValue as usize)
    }
    /// Borrow this `ValId<P>` as a `ValRef`
    pub fn borrow_val(&self) -> ValRef {
        ValRef {
            ptr: self.ptr.borrow_arc(),
            variant: PhantomData,
        }
    }
    /// Borrow this `ValId<P>` as a `ValRef<P>`
    pub fn borrow_var(&self) -> ValRef<P> {
        ValRef {
            ptr: self.ptr.borrow_arc(),
            variant: self.variant,
        }
    }
    /// Clone this `ValId<P>` as a `ValId`
    pub fn clone_val(&self) -> ValId {
        self.clone().coerce()
    }
    /// Coerce this `ValId` into another predicated value
    #[inline]
    pub(crate) fn coerce<Q>(self) -> ValId<Q> {
        ValId {
            ptr: unsafe { std::mem::transmute(self) },
            variant: PhantomData,
        }
    }
    /// Coerce this `ValId` into a reference to another predicated value
    #[inline]
    pub(crate) fn coerce_ref<Q>(&self) -> &ValId<Q> {
        let ptr_ref = &self.ptr;
        unsafe { &*(ptr_ref as *const _ as *const ValId<Q>) }
    }
}

// General borrowed value methods

impl<'a, P> ValRef<'a, P> {
    /// Get this `ValRef<P>` as a `NormalValue`
    pub fn as_norm(self) -> &'a NormalValue {
        self.ptr.get()
    }
    /// Get this `ValRef<P>` as a `ValueEnum`
    pub fn as_enum(self) -> &'a ValueEnum {
        self.ptr.get()
    }
    /// Get this `ValRef<P>` as a `NormalValue<P>`
    #[inline]
    pub fn as_pred(self) -> &'a NormalValue<P> {
        self.ptr.get().coerce_ref()
    }
    /// Get this `ValRef<P>` as a `ValRef`
    pub fn as_val(self) -> ValRef<'a> {
        ValRef {
            ptr: self.ptr,
            variant: PhantomData,
        }
    }
    /// Get this `ValRef<P>` as a `ValId<P>`
    pub fn as_var(&self) -> &ValId<P> {
        let arc_ptr = self.ptr.as_arc();
        unsafe { &*(arc_ptr as *const _ as *const ValId<P>) }
    }
    /// Get this `ValRef<P>` as a `ValId`
    pub fn as_valid(&self) -> &ValId {
        self.as_var().as_val()
    }
    /// Clone this `ValRef<P>` as a `ValId`
    pub fn clone_val(self) -> ValId {
        ValId {
            ptr: self.ptr.clone_arc(),
            variant: PhantomData,
        }
    }
    /// Clone this `ValRef<P>` as a `ValId<P>`
    pub fn clone_var(self) -> ValId<P> {
        ValId {
            ptr: self.ptr.clone_arc(),
            variant: self.variant,
        }
    }
    /// Get the pointer behind this `ValRef`
    #[inline]
    pub fn as_ptr(self) -> *const NormalValue {
        self.as_norm() as *const NormalValue
    }
    /// Coerce a pointer into a `ValRef`
    ///
    /// # Safety
    /// This function can only be called with the result of `as_ptr` for `ValId` or `ValRef`, or
    /// `into_ptr` for `ValId`.
    #[inline]
    pub unsafe fn from_raw(ptr: *const NormalValue) -> ValRef<'a, P> {
        ValRef {
            ptr: ArcBorrow::from_raw(ptr),
            variant: PhantomData,
        }
    }
    /// Get the address behind this `ValRef`
    #[inline]
    pub fn as_addr(self) -> ValAddr {
        ValAddr(self.as_norm() as *const NormalValue as usize)
    }
    /// Coerce this reference
    #[inline]
    pub(crate) fn coerce<Q>(self) -> ValRef<'a, Q> {
        ValRef {
            ptr: self.ptr,
            variant: PhantomData,
        }
    }
}

// Value implementation

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
        self.coerce()
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

// Value sub-trait implementations

impl<P> Typed for ValId<P> {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.as_enum().ty()
    }
    #[inline]
    fn kind(&self) -> KindRef {
        self.as_enum().kind()
    }
    #[inline]
    fn repr(&self) -> Option<ReprRef> {
        self.as_enum().repr()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        self.as_enum().is_ty()
    }
    #[inline]
    fn is_kind(&self) -> bool {
        self.as_enum().is_kind()
    }
    #[inline]
    fn is_repr(&self) -> bool {
        self.as_enum().is_repr()
    }
    #[inline]
    fn is_universe(&self) -> bool {
        self.as_enum().is_universe()
    }
    #[inline]
    fn kind_level(&self) -> usize {
        self.as_enum().kind_level()
    }
}

impl<P> Typed for ValRef<'_, P> {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.as_enum().ty()
    }
    #[inline]
    fn kind(&self) -> KindRef {
        self.as_enum().kind()
    }
    #[inline]
    fn repr(&self) -> Option<ReprRef> {
        self.as_enum().repr()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        self.as_enum().is_ty()
    }
    #[inline]
    fn is_kind(&self) -> bool {
        self.as_enum().is_kind()
    }
    #[inline]
    fn is_repr(&self) -> bool {
        self.as_enum().is_repr()
    }
    #[inline]
    fn is_universe(&self) -> bool {
        self.as_enum().is_universe()
    }
    #[inline]
    fn kind_level(&self) -> usize {
        self.as_enum().kind_level()
    }
}

impl<P> Apply for ValId<P> {
    #[inline]
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        self.as_norm().apply_in(args, ctx)
    }
}

impl<P> Apply for ValRef<'_, P> {
    #[inline]
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        self.as_norm().apply_in(args, ctx)
    }
}

impl<P> Substitute<ValId> for ValId<P> {
    #[inline]
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<ValId, Error> {
        self.as_val().substitute_impl(ctx)
    }
}

impl<P> Substitute<ValId> for ValRef<'_, P> {
    #[inline]
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<ValId, Error> {
        self.clone_val().substitute(ctx)
    }
}

impl<P> Regional for ValId<P> {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.as_norm().region()
    }
    #[inline]
    fn depth(&self) -> usize {
        self.as_norm().depth()
    }
}

impl<P> Regional for ValRef<'_, P> {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.as_norm().region()
    }
    #[inline]
    fn depth(&self) -> usize {
        self.as_norm().depth()
    }
}

// VarId construction and methods

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
            ptr: VALUE_CACHE.cache(norm),
            variant: PhantomData,
        }
    }
}

// Borrowing and dereferencing

impl Deref for ValId {
    type Target = NormalValue;
    #[inline]
    fn deref(&self) -> &NormalValue {
        &self.ptr
    }
}

impl<'a> Deref for ValRef<'_> {
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

impl<P> Borrow<ValId> for ValRef<'_, P> {
    #[inline]
    fn borrow(&self) -> &ValId {
        self.as_valid()
    }
}

impl<P> Borrow<VarId<P>> for VarRef<'_, P> {
    #[inline]
    fn borrow(&self) -> &VarId<P> {
        self.as_var()
    }
}

impl<P> Borrow<NormalValue> for ValRef<'_, P> {
    #[inline]
    fn borrow(&self) -> &NormalValue {
        &self.ptr
    }
}

impl<P> Borrow<ValueEnum> for ValRef<'_, P> {
    #[inline]
    fn borrow(&self) -> &ValueEnum {
        &self.ptr
    }
}

// ValId construction and casting

impl From<NormalValue> for ValId {
    #[inline]
    fn from(value: NormalValue) -> ValId {
        ValId {
            ptr: VALUE_CACHE.cache(value),
            variant: PhantomData,
        }
    }
}

impl From<Arc<NormalValue>> for ValId {
    #[inline]
    fn from(value: Arc<NormalValue>) -> ValId {
        ValId {
            ptr: VALUE_CACHE.cache(value),
            variant: PhantomData,
        }
    }
}

impl From<TypeId> for ValId {
    fn from(ty: TypeId) -> ValId {
        ty.coerce()
    }
}

impl<V> From<VarId<V>> for ValId {
    fn from(v: VarId<V>) -> ValId {
        v.coerce()
    }
}

impl<V> TryFrom<ValId> for VarId<V>
where
    for<'a> &'a NormalValue: TryInto<&'a V>,
{
    type Error = ValId;
    fn try_from(v: ValId) -> Result<VarId<V>, ValId> {
        if TryInto::<&V>::try_into(v.as_norm()).is_ok() {
            Ok(v.coerce())
        } else {
            Err(v)
        }
    }
}

impl TryFrom<ValId> for TypeId {
    type Error = ValId;
    fn try_from(v: ValId) -> Result<TypeId, ValId> {
        if v.is_ty() {
            Ok(v.coerce())
        } else {
            Err(v)
        }
    }
}

impl TryFrom<ValId> for KindId {
    type Error = ValId;
    fn try_from(v: ValId) -> Result<KindId, ValId> {
        if v.is_kind() {
            Ok(v.coerce())
        } else {
            Err(v)
        }
    }
}

impl<V: ValueData> From<V> for VarId<V> {
    fn from(v: V) -> VarId<V> {
        v.into_val().coerce()
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
                variant: PhantomData,
            })
        } else {
            Err(v)
        }
    }
}

impl<'a> From<&'a TypeId> for &'a ValId {
    fn from(ty: &'a TypeId) -> &'a ValId {
        ty.coerce_ref()
    }
}

impl<'a, V: Value> From<&'a VarId<V>> for &'a ValId {
    fn from(var: &'a VarId<V>) -> &'a ValId {
        var.coerce_ref()
    }
}

impl<P> From<ValRef<'_, P>> for ValId {
    fn from(val: ValRef<P>) -> ValId {
        val.clone_val()
    }
}

// Construction from ValIds

impl<P> From<ValId<P>> for ValueEnum {
    fn from(value: ValId<P>) -> ValueEnum {
        value.into_enum()
    }
}

impl<P> From<ValId<P>> for NormalValue {
    fn from(value: ValId<P>) -> NormalValue {
        value.into_norm()
    }
}

// Debug, display and prettyprinting

impl<P> Debug for ValId<P> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(self.as_norm(), fmt)
    }
}

impl<P> Debug for ValRef<'_, P> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(self.as_norm(), fmt)
    }
}

impl<P> Display for ValId<P> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        Display::fmt(self.as_norm(), fmt)
    }
}

impl<P> Display for ValRef<'_, P> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        Display::fmt(self.as_norm(), fmt)
    }
}

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Formatter};

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
}
