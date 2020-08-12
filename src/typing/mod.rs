/*!
The `rain` type system
*/
use super::{
    eval::EvalCtx,
    lifetime::Lifetime,
    value::{
        Error, KindRef, ReprRef, TypeId, TypeRef, UniverseRef, ValId, ValRef, Value, ValueEnum,
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
    fn is_ty(&self) -> bool;
    /// Check whether this `rain` value is a kind
    fn is_kind(&self) -> bool;
    /// Check whether this `rain` value is a representation
    #[inline]
    fn is_repr(&self) -> bool {
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
    fn into_ty(self) -> TypeId {
        self.into_val().coerce()
    }
    /// Get the kind of this type
    ///
    /// # Correctness
    /// The result of this method *must* be pointer-equivalent to the result of calling `.ty()` on this type
    fn ty_kind(&self) -> KindRef {
        let ty = self.ty();
        debug_assert!(ty.is_kind(), "The type of a type must be a kind!");
        ty.coerce()
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
    /// Get the universe of this type
    ///
    /// # Notes
    /// The result of this method *might* not be equal to the result of calling `.ty()` on this type.
    /// Specifically, the universe of a type must be a supertype of it's type, but does not have to *be*
    /// it's type.
    fn universe(&self) -> UniverseRef;
    /// Get whether this type is a universe
    fn is_universe(&self) -> bool;
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
    /// for an empty argument vector, and an error otherwise.
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
    /// Substitute this value while preserving the fact that it is a type
    fn substitute_ty(&self, ctx: &mut EvalCtx) -> Result<TypeId, Error> {
        let value = self.substitute(ctx)?;
        value.try_into().map_err(|_| Error::NotATypeError)
    }
}

impl<P: TypePredicate> Type for ValId<P> {
    fn into_ty(self) -> TypeId {
        self.coerce()
    }
    fn universe(&self) -> UniverseRef {
        match self.as_enum() {
            ValueEnum::BoolTy(b) => b.universe(),
            ValueEnum::Finite(f) => f.universe(),
            ValueEnum::Pi(p) => p.universe(),
            ValueEnum::Universe(u) => u.universe(),
            ValueEnum::Product(p) => p.universe(),
            ValueEnum::Parameter(p) => unimplemented!("Parameter universes for parameter {}", p),
            ValueEnum::Sexpr(s) => unimplemented!("Partial evaluation universes for sexpr {}", s),
            v => panic!(
                "Logic error: value {} is not a type, but was asserted as such!",
                v
            ),
        }
    }
    fn is_universe(&self) -> bool {
        match self.as_enum() {
            ValueEnum::BoolTy(b) => b.is_universe(),
            ValueEnum::Finite(f) => f.is_universe(),
            ValueEnum::Pi(p) => p.is_universe(),
            ValueEnum::Universe(u) => u.is_universe(),
            ValueEnum::Product(p) => p.is_universe(),
            ValueEnum::Parameter(p) => {
                unimplemented!("Parameter universe check for parameter {}", p)
            }
            ValueEnum::Sexpr(s) => {
                unimplemented!("Partial evaluation universe check for sexpr {}", s)
            }
            v => panic!(
                "Logic error: value {} is not a type, but was asserted as such!",
                v
            ),
        }
    }
    fn is_affine(&self) -> bool {
        match self.as_enum() {
            ValueEnum::BoolTy(b) => b.is_affine(),
            ValueEnum::Finite(f) => f.is_affine(),
            ValueEnum::Pi(p) => p.is_affine(),
            ValueEnum::Universe(u) => u.is_affine(),
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
    fn is_relevant(&self) -> bool {
        match self.as_enum() {
            ValueEnum::BoolTy(b) => b.is_relevant(),
            ValueEnum::Finite(f) => f.is_relevant(),
            ValueEnum::Pi(p) => p.is_relevant(),
            ValueEnum::Universe(u) => u.is_relevant(),
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
            ValueEnum::Universe(u) => u.is_linear(),
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
            ValueEnum::Universe(u) => u.is_substruct(),
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
    fn apply_ty(&self, args: &[ValId]) -> Result<(Lifetime, TypeId), Error>
    where
        Self: Clone,
    {
        match self.as_enum() {
            ValueEnum::BoolTy(b) => b.apply_ty(args),
            ValueEnum::Finite(f) => f.apply_ty(args),
            ValueEnum::Pi(p) => p.apply_ty(args),
            ValueEnum::Universe(u) => u.apply_ty(args),
            ValueEnum::Product(p) => p.apply_ty(args),
            ValueEnum::Parameter(p) => unimplemented!("Parameter application for parameter {}", p),
            ValueEnum::Sexpr(s) => unimplemented!("Partial evaluation application for sexpr {}", s),
            v => panic!(
                "Logic error: value {} is not a type, but was asserted as such!",
                v
            ),
        }
    }
    fn apply_ty_in(
        &self,
        args: &[ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<(Lifetime, TypeId), Error>
    where
        Self: Clone,
    {
        match self.as_enum() {
            ValueEnum::BoolTy(b) => b.apply_ty_in(args, ctx),
            ValueEnum::Finite(f) => f.apply_ty_in(args, ctx),
            ValueEnum::Pi(p) => p.apply_ty_in(args, ctx),
            ValueEnum::Universe(u) => u.apply_ty_in(args, ctx),
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

impl<'a, P: TypePredicate> Type for ValRef<'a, P> {
    fn universe(&self) -> UniverseRef {
        match self.as_enum() {
            ValueEnum::BoolTy(b) => b.universe(),
            ValueEnum::Finite(f) => f.universe(),
            ValueEnum::Pi(p) => p.universe(),
            ValueEnum::Universe(u) => u.universe(),
            ValueEnum::Product(p) => p.universe(),
            ValueEnum::Parameter(p) => unimplemented!("Parameter universes for parameter {}", p),
            ValueEnum::Sexpr(s) => unimplemented!("Partial evaluation universes for sexpr {}", s),
            v => panic!(
                "Logic error: value {} is not a type, but was asserted as such!",
                v
            ),
        }
    }
    fn is_universe(&self) -> bool {
        match self.as_enum() {
            ValueEnum::BoolTy(b) => b.is_universe(),
            ValueEnum::Finite(f) => f.is_universe(),
            ValueEnum::Pi(p) => p.is_universe(),
            ValueEnum::Universe(u) => u.is_universe(),
            ValueEnum::Product(p) => p.is_universe(),
            ValueEnum::Parameter(p) => {
                unimplemented!("Parameter universe check for parameter {}", p)
            }
            ValueEnum::Sexpr(s) => {
                unimplemented!("Partial evaluation universe check for sexpr {}", s)
            }
            v => panic!(
                "Logic error: value {} is not a type, but was asserted as such!",
                v
            ),
        }
    }
    fn is_affine(&self) -> bool {
        match self.as_enum() {
            ValueEnum::BoolTy(b) => b.is_affine(),
            ValueEnum::Finite(f) => f.is_affine(),
            ValueEnum::Pi(p) => p.is_affine(),
            ValueEnum::Universe(u) => u.is_affine(),
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
    fn is_relevant(&self) -> bool {
        match self.as_enum() {
            ValueEnum::BoolTy(b) => b.is_relevant(),
            ValueEnum::Finite(f) => f.is_relevant(),
            ValueEnum::Pi(p) => p.is_relevant(),
            ValueEnum::Universe(u) => u.is_relevant(),
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
            ValueEnum::Universe(u) => u.is_linear(),
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
            ValueEnum::Universe(u) => u.is_substruct(),
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
    fn apply_ty(&self, args: &[ValId]) -> Result<(Lifetime, TypeId), Error>
    where
        Self: Clone,
    {
        match self.as_enum() {
            ValueEnum::BoolTy(b) => b.apply_ty(args),
            ValueEnum::Finite(f) => f.apply_ty(args),
            ValueEnum::Pi(p) => p.apply_ty(args),
            ValueEnum::Universe(u) => u.apply_ty(args),
            ValueEnum::Product(p) => p.apply_ty(args),
            ValueEnum::Parameter(p) => unimplemented!("Parameter application for parameter {}", p),
            ValueEnum::Sexpr(s) => unimplemented!("Partial evaluation application for sexpr {}", s),
            v => panic!(
                "Logic error: value {} is not a type, but was asserted as such!",
                v
            ),
        }
    }
    fn apply_ty_in(
        &self,
        args: &[ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<(Lifetime, TypeId), Error>
    where
        Self: Clone,
    {
        match self.as_enum() {
            ValueEnum::BoolTy(b) => b.apply_ty_in(args, ctx),
            ValueEnum::Finite(f) => f.apply_ty_in(args, ctx),
            ValueEnum::Pi(p) => p.apply_ty_in(args, ctx),
            ValueEnum::Universe(u) => u.apply_ty_in(args, ctx),
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
