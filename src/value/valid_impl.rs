use super::*;

// Statics

lazy_static! {
    /// The global `rain` value cache
    pub static ref VALUE_CACHE: Cache<NormalValue> = Cache::new();
}

// Equality

impl<P> Eq for ValId<P> {}

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

// Base methods (implemented on unqualified `ValId` pointers)

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

// General methods (implemented on all `ValId` pointers)

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
    pub(super) fn coerce<Q>(self) -> ValId<Q> {
        ValId {
            ptr: self.ptr,
            variant: std::marker::PhantomData,
        }
    }
    /// Coerce this `ValId` into a reference to another predicated value
    pub(super) fn coerce_ref<Q>(&self) -> &ValId<Q> {
        RefCast::ref_cast(&self.ptr)
    }
}

// General borrowed value methods

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

// Borrowed variant methods

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
        self.as_norm().ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        self.as_norm().is_ty()
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

impl<P> Apply for ValId<P> {
    #[inline]
    fn do_apply<'a>(&self, args: &'a [ValId], inline: bool) -> Result<Application<'a>, Error> {
        self.as_norm().do_apply(args, inline)
    }
}

impl<P> Apply for ValRef<'_, P> {
    #[inline]
    fn do_apply<'a>(&self, args: &'a [ValId], inline: bool) -> Result<Application<'a>, Error> {
        self.as_norm().do_apply(args, inline)
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
}

impl<P> Regional for ValRef<'_, P> {
    #[inline]
    fn region(&self) -> RegionBorrow {
        self.as_norm().region()
    }
}

impl<P> Live for ValId<P> {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.as_norm().lifetime()
    }
}

impl<P> Live for ValRef<'_, P> {
    #[inline]
    fn lifetime(&self) -> LifetimeBorrow {
        self.as_norm().lifetime()
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


// ValId construction and casting

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

impl<V> From<V> for VarId<V>
where
    V: Into<ValId>,
    for<'a> &'a NormalValue: TryInto<&'a V>,
{
    fn from(val: V) -> VarId<V> {
        let valid: ValId = val.into();
        valid.coerce()
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

impl<'a, V: Value> From<&'a VarId<V>> for &'a ValId {
    fn from(var: &'a VarId<V>) -> &'a ValId {
        RefCast::ref_cast(&var.ptr)
    }
}

impl<V: Value> From<VarRef<'_, V>> for ValId {
    fn from(val: VarRef<V>) -> ValId {
        val.clone_val()
    }
}

impl From<ValRef<'_>> for ValId {
    fn from(val: ValRef) -> ValId {
        val.clone_val()
    }
}

// Construction from ValIds

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

// Debug, display and prettyprinting

debug_from_display!(ValId);
pretty_display!(ValId, s, fmt  => write!(fmt, "{}", s.deref()));
debug_from_display!(ValRef<'_>);
pretty_display!(ValRef<'_>, s, fmt  => write!(fmt, "{}", s.deref()));

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