/*!
The `rain` type system
*/
use super::{
    eval::EvalCtx,
    lifetime::Lifetime,
    value::{Error, TypeId, TypeRef, UniverseRef, ValId, ValRef, Value, ValueEnum},
};
use std::convert::TryInto;

mod kind;
pub use kind::*;
mod predicate;
pub use predicate::*;

/// A trait implemented by `rain` values with a type
pub trait Typed {
    /// Compute the type of this `rain` value
    fn ty(&self) -> TypeRef;
    /// Check whether this `rain` value is a type
    fn is_ty(&self) -> bool;
    /// Check whether this `rain` value is a kind
    fn is_kind(&self) -> bool;
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
