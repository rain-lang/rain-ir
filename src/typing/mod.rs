/*!
The `rain` type system
*/
use super::{
    eval::EvalCtx,
    lifetime::Lifetime,
    value::{
        Error, KindRef, NormalValue, ReprRef, TypeId, TypeRef, UniverseRef, ValId, ValRef, Value,
        ValueEnum,
    },
};
use std::convert::TryInto;

mod kind;
pub use kind::*;
mod predicate;
pub use predicate::*;

/// A trait implemented by `rain` values with a type
pub trait Typed {
    /// Compute the type of this `rain` value
    ///
    /// # Example
    /// ```rust
    /// # use rain_ir::{typing::Typed, primitive::logical::Bool, value::Value};
    /// let bool_ty = Bool.into_val();
    /// assert_eq!(true.ty(), bool_ty);
    /// assert_eq!(false.ty(), bool_ty);
    /// ```
    fn ty(&self) -> TypeRef;
    /// Compute the kind of this `rain` value
    fn kind(&self) -> KindRef {
        let tyty = self.ty().as_enum().ty();
        debug_assert!(tyty.is_kind(), "The type of a type must be a kind!");
        tyty.coerce()
    }
    /// Compute the representation of this `rain` value, if any
    fn repr(&self) -> Option<ReprRef> {
        let tyty = self.ty().as_enum().ty();
        if tyty.is_repr() {
            Some(tyty.coerce())
        } else {
            None
        }
    }
    /// Check whether this `rain` value is a type
    ///
    /// # Example
    /// ```rust
    /// # use rain_ir::{typing::Typed, primitive::logical::Bool, value::Value};
    /// assert!(!true.is_ty());
    /// assert!(Bool.is_ty());
    /// ```
    fn is_ty(&self) -> bool {
        false
    }
    /// Check whether this `rain` value is a kind
    ///
    /// # Correctness
    /// If a value is a kind, it must *always* be a type.
    fn is_kind(&self) -> bool {
        false
    }
    /// Check whether this `rain` value is a representation
    ///
    /// # Correctness
    /// If a value is a representation, it must *always* be a kind.
    #[inline]
    fn is_repr(&self) -> bool {
        false
    }
    /// Check whether this `rain` value is a universe
    ///
    /// # Correctness
    /// If a value is a universe, it must *always* be a kind.
    #[inline]
    fn is_universe(&self) -> bool {
        false
    }
    /// Get the kind-level of this value
    ///
    /// We define kind-level inductively as follows:
    /// - A level-0 value is just a value
    /// - A level-(n + 1) value is a type whose elements are all of level-n
    ///
    /// A level-1 value is called a *type*, while a level-2 value is called a *kind*.
    /// A level-3 value is sometimes called a *sort*.
    ///
    /// # Correctness
    /// This method may return an under-estimate, but may *not* return an over-estimate
    #[inline]
    fn kind_level(&self) -> usize {
        if self.is_ty() {
            if self.is_kind() {
                2
            } else {
                0
            }
        } else {
            debug_assert!(
                !self.is_kind(),
                "A value cannot be a kind if it is not a type!"
            );
            1
        }
    }
}

/// A trait implemented by `rain` values which are a type
pub trait Type: Value {
    /// Convert this type into a `TypeId`
    ///
    /// # Correctness
    /// The result of this method must *always* be pointer-equivalent to the result of the `.into_val()` method of
    /// the `Value` trait.
    #[inline]
    fn into_ty(self) -> TypeId {
        self.into_val().coerce()
    }
    /// Get the kind of this type
    ///
    /// # Correctness
    /// The result of this method *must* be pointer-equivalent to the result of calling `.ty()` on this type
    #[inline]
    fn ty_kind(&self) -> KindRef {
        let ty = self.ty();
        debug_assert!(ty.is_kind(), "The type of a type must be a kind!");
        ty.coerce()
    }
    /// Get the universe of this type
    #[inline]
    fn universe(&self) -> UniverseRef {
        self.ty_kind().get_closure()
    }
    /// Get the representation of this type, if any
    fn ty_repr(&self) -> Option<ReprRef> {
        let ty = self.ty();
        if ty.is_repr() {
            Some(ty.coerce())
        } else {
            None
        }
    }
    /// Get whether this type is affine
    fn is_affine(&self) -> bool;
    /// Get whether this type is relevant
    fn is_relevant(&self) -> bool;
    /// Get whether this type is linear
    ///
    /// # Correctness
    /// A type is linear if and only if it is both affine and relevant.
    /// It is recommended to use the default implementation for this method unless there
    /// is a more efficient one available.
    #[inline]
    fn is_linear(&self) -> bool {
        self.is_affine() && self.is_relevant()
    }
    /// Get whether this type is substructural
    ///
    /// # Correctness
    /// A type is substructural if and only if it is either affine or relevant
    /// It is recommended to use the default implementation for this method unless there
    /// is a more efficient one available.
    #[inline]
    fn is_substruct(&self) -> bool {
        self.is_affine() || self.is_relevant()
    }
    /// Apply this type to a set of arguments, yielding a result type and lifetime
    ///
    /// # Correctness
    /// This method must always return the same value as calling `self.apply_ty_in(args, &mut None)`.
    /// The default implementation does exactly this, and in general should be used as is.
    fn apply_ty(&self, args: &[ValId]) -> Result<(Lifetime, TypeId), Error>
    where
        Self: Clone,
    {
        self.apply_ty_in(args, &mut None)
    }
    /// Apply this type to a set of arguments, yielding a result type and lifetime
    ///
    /// # Notes
    /// The default implementation for this method returns this value and it's lifetime
    /// for an empty argument vector, and a `NotAFunctionType` error otherwise. It is appropriate
    /// for types which are, as the error indicates, not function types.
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
            Err(Error::NotAFunctionType)
        }
    }
    /// Substitute this value while preserving the fact that it is a type
    fn substitute_ty(&self, ctx: &mut EvalCtx) -> Result<TypeId, Error> {
        let value = self.substitute(ctx)?;
        value.try_into().map_err(|_| Error::NotATypeError)
    }
}

impl<P: TypePredicate> Type for ValId<P> {
    #[inline]
    fn into_ty(self) -> TypeId {
        debug_assert!(self.is_ty());
        self.coerce()
    }
    #[inline]
    fn is_affine(&self) -> bool {
        self.as_pred().is_affine()
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        self.as_pred().is_relevant()
    }
    #[inline]
    fn is_linear(&self) -> bool {
        self.as_pred().is_linear()
    }
    #[inline]
    fn is_substruct(&self) -> bool {
        self.as_pred().is_substruct()
    }
    #[inline]
    fn apply_ty(&self, args: &[ValId]) -> Result<(Lifetime, TypeId), Error> {
        self.as_pred().apply_ty(args)
    }
    #[inline]
    fn apply_ty_in(
        &self,
        args: &[ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<(Lifetime, TypeId), Error> {
        self.as_pred().apply_ty_in(args, ctx)
    }
}

impl<'a, P: TypePredicate> Type for ValRef<'a, P> {
    #[inline]
    fn is_affine(&self) -> bool {
        self.as_pred().is_affine()
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        self.as_pred().is_relevant()
    }
    #[inline]
    fn is_linear(&self) -> bool {
        self.as_pred().is_linear()
    }
    #[inline]
    fn is_substruct(&self) -> bool {
        self.as_pred().is_substruct()
    }
    #[inline]
    fn apply_ty(&self, args: &[ValId]) -> Result<(Lifetime, TypeId), Error> {
        self.as_pred().apply_ty(args)
    }
    #[inline]
    fn apply_ty_in(
        &self,
        args: &[ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<(Lifetime, TypeId), Error> {
        self.as_pred().apply_ty_in(args, ctx)
    }
}

impl<'a, P: TypePredicate> Type for NormalValue<P> {
    #[inline]
    fn is_affine(&self) -> bool {
        match self.as_enum() {
            ValueEnum::BoolTy(b) => b.is_affine(),
            ValueEnum::Finite(f) => f.is_affine(),
            ValueEnum::Pi(p) => p.is_affine(),
            ValueEnum::Prop(u) => u.is_affine(),
            ValueEnum::Fin(u) => u.is_affine(),
            ValueEnum::Set(u) => u.is_affine(),
            ValueEnum::Product(p) => p.is_affine(),
            ValueEnum::Parameter(p) => {
                unimplemented!("Parameter affinity check for parameter {}", p)
            }
            ValueEnum::Sexpr(s) => {
                unimplemented!("Partial evaluation affinity check for sexpr {}", s)
            }
            v => panic!(
                "Logic error: value {} is not a type, but was asserted as such!",
                v
            ),
        }
    }
    #[inline]
    fn is_relevant(&self) -> bool {
        match self.as_enum() {
            ValueEnum::BoolTy(b) => b.is_relevant(),
            ValueEnum::Finite(f) => f.is_relevant(),
            ValueEnum::Pi(p) => p.is_relevant(),
            ValueEnum::Prop(u) => u.is_relevant(),
            ValueEnum::Fin(u) => u.is_relevant(),
            ValueEnum::Set(u) => u.is_relevant(),
            ValueEnum::Product(p) => p.is_relevant(),
            ValueEnum::Parameter(p) => {
                unimplemented!("Parameter relevance check for parameter {}", p)
            }
            ValueEnum::Sexpr(s) => {
                unimplemented!("Partial evaluation relevance check for sexpr {}", s)
            }
            v => panic!(
                "Logic error: value {} is not a type, but was asserted as such!",
                v
            ),
        }
    }
    #[inline]
    fn is_linear(&self) -> bool {
        match self.as_enum() {
            ValueEnum::BoolTy(b) => b.is_linear(),
            ValueEnum::Finite(f) => f.is_linear(),
            ValueEnum::Pi(p) => p.is_linear(),
            ValueEnum::Prop(u) => u.is_linear(),
            ValueEnum::Fin(u) => u.is_linear(),
            ValueEnum::Set(u) => u.is_linear(),
            ValueEnum::Product(p) => p.is_linear(),
            ValueEnum::Parameter(p) => {
                unimplemented!("Parameter linearity check for parameter {}", p)
            }
            ValueEnum::Sexpr(s) => {
                unimplemented!("Partial evaluation linearity check for sexpr {}", s)
            }
            v => panic!(
                "Logic error: value {} is not a type, but was asserted as such!",
                v
            ),
        }
    }
    #[inline]
    fn is_substruct(&self) -> bool {
        match self.as_enum() {
            ValueEnum::BoolTy(b) => b.is_substruct(),
            ValueEnum::Finite(f) => f.is_substruct(),
            ValueEnum::Pi(p) => p.is_substruct(),
            ValueEnum::Prop(u) => u.is_substruct(),
            ValueEnum::Fin(u) => u.is_substruct(),
            ValueEnum::Set(u) => u.is_substruct(),
            ValueEnum::Product(p) => p.is_substruct(),
            ValueEnum::Parameter(p) => {
                unimplemented!("Parameter substructurality check for parameter {}", p)
            }
            ValueEnum::Sexpr(s) => {
                unimplemented!("Partial evaluation substructurality check for sexpr {}", s)
            }
            v => panic!(
                "Logic error: value {} is not a type, but was asserted as such!",
                v
            ),
        }
    }
    #[inline]
    fn apply_ty(&self, args: &[ValId]) -> Result<(Lifetime, TypeId), Error> {
        match self.as_enum() {
            ValueEnum::BoolTy(b) => b.apply_ty(args),
            ValueEnum::Finite(f) => f.apply_ty(args),
            ValueEnum::Pi(p) => p.apply_ty(args),
            ValueEnum::Prop(u) => u.apply_ty(args),
            ValueEnum::Fin(u) => u.apply_ty(args),
            ValueEnum::Set(u) => u.apply_ty(args),
            ValueEnum::Product(p) => p.apply_ty(args),
            ValueEnum::Parameter(p) => unimplemented!("Parameter application for parameter {}", p),
            ValueEnum::Sexpr(s) => unimplemented!("Partial evaluation application for sexpr {}", s),
            v => panic!(
                "Logic error: value {} is not a type, but was asserted as such!",
                v
            ),
        }
    }
    #[inline]
    fn apply_ty_in(
        &self,
        args: &[ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<(Lifetime, TypeId), Error> {
        match self.as_enum() {
            ValueEnum::BoolTy(b) => b.apply_ty_in(args, ctx),
            ValueEnum::Finite(f) => f.apply_ty_in(args, ctx),
            ValueEnum::Pi(p) => p.apply_ty_in(args, ctx),
            ValueEnum::Prop(u) => u.apply_ty_in(args, ctx),
            ValueEnum::Fin(u) => u.apply_ty_in(args, ctx),
            ValueEnum::Set(u) => u.apply_ty_in(args, ctx),
            ValueEnum::Product(p) => p.apply_ty_in(args, ctx),
            ValueEnum::Parameter(p) => {
                unimplemented!("Parameter contextual application for parameter {}", p)
            }
            ValueEnum::Sexpr(s) => {
                unimplemented!("Partial evaluation contextual application for sexpr {}", s)
            }
            v => panic!(
                "Logic error: value {} is not a type, but was asserted as such!",
                v
            ),
        }
    }
}

impl<P: TypePredicate> ValId<P> {
    /// Get this `ValId` as a `TypeId`
    #[inline(always)]
    pub fn as_ty(&self) -> &TypeId {
        self.coerce_ref()
    }
    /// Clone this `ValId` as a `TypeId`
    #[inline(always)]
    pub fn clone_ty(&self) -> TypeId {
        self.as_ty().clone()
    }
    /// Borrow this `ValId` as a `TypeRef`
    #[inline(always)]
    pub fn borrow_ty(&self) -> TypeRef {
        self.as_ty().borrow_var()
    }
}
