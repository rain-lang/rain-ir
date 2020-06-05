/*!
`rain` expressions
*/
use super::{
    eval::{Application, Apply, Error, EvalCtx, Substitute, SubstituteToValId},
    lifetime::{Lifetime, LifetimeBorrow, Live},
    primitive::UNIT_TY,
    typing::{Type, Typed},
    TypeId, TypeRef, ValId, Value, ValueEnum,
};
use crate::{debug_from_display, pretty_display};
use smallvec::{smallvec, SmallVec};
use std::ops::Deref;

/// The size of a small S-expression
pub const SMALL_SEXPR_SIZE: usize = 3;

/// The argument-vector of an S-expression
pub type SexprArgs = SmallVec<[ValId; SMALL_SEXPR_SIZE]>;

/// An S-expression
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Sexpr {
    /// The arguments of this S-expression
    args: SexprArgs,
    /// The (cached) lifetime of this S-expression
    lifetime: Lifetime,
    /// The (cached) type of this S-expression
    ty: TypeId,
}

debug_from_display!(Sexpr);
pretty_display!(Sexpr, "(...)");

impl Sexpr {
    /// Attempt to create an S-expression from an owned argument list, evaluating as necessary.
    pub fn try_new(mut args: SexprArgs) -> Result<Sexpr, Error> {
        // Simple cases
        match args.len() {
            0 => return Ok(Self::unit()),
            1 => return Ok(Self::singleton(args.swap_remove(0))),
            _ => {}
        }
        // Expand sexprs in the first argument
        match args[0].as_enum() {
            ValueEnum::Sexpr(s) => {
                if s.len() == 0 {
                    return Err(Error::EmptySexprApp); // Special error for unit application
                }
                let mut new_args = SexprArgs::with_capacity(args.len() + s.len());
                new_args.extend(s.iter().cloned());
                new_args.extend(args.drain(1..));
                args = new_args;
            }
            _ => {}
        }
        // General case
        let (lifetime, ty) = match args[0].apply(&args[1..])? {
            Application::Success(rest, valid) => return Self::applied_with(valid, rest),
            Application::Complete(lifetime, ty)
            | Application::Incomplete(lifetime, ty)
            | Application::Stop(lifetime, ty) => (lifetime, ty),
        };
        Ok(Sexpr { args, lifetime, ty })
    }
    /// Attempt to create an S-expression from an un-owned argument-list, evaluating as necessary
    pub fn eval(args: &[ValId]) -> Result<Sexpr, Error> {
        match args.len() {
            0 => return Ok(Self::unit()),
            _ => Self::applied_with(args[0].clone(), &args[..]),
        }
    }
    /// Attempt to create an S-expression by applying an argument to an argument list, evaluating as necessary.
    pub fn applied_with(mut f: ValId, mut args: &[ValId]) -> Result<Sexpr, Error> {
        while !args.is_empty() {
            match f.apply(args)? {
                Application::Success(rest, v) => {
                    args = rest;
                    f = v;
                    continue;
                }
                Application::Complete(lifetime, ty)
                | Application::Incomplete(lifetime, ty)
                | Application::Stop(lifetime, ty) => {
                    let mut a = SexprArgs::with_capacity(1 + args.len());
                    a.push(f);
                    a.clone_from_slice(args);
                    return Ok(Sexpr {
                        args: a,
                        lifetime,
                        ty,
                    });
                }
            };
        }
        Ok(Self::singleton(f))
    }
    /// Create an S-expression corresponding to the unit value
    pub fn unit() -> Sexpr {
        Sexpr {
            args: SexprArgs::new(),
            lifetime: Lifetime::default(),
            ty: UNIT_TY.as_ty().clone(),
        }
    }
    /// Create an S-expression corresponding to a singleton value
    pub fn singleton(value: ValId) -> Sexpr {
        if let ValueEnum::Sexpr(s) = value.as_enum() {
            // Edge case
            return s.clone();
        }
        let ty = value.ty().clone_ty();
        Sexpr {
            args: smallvec![value],
            lifetime: Lifetime::default(),
            ty,
        }
    }
}

impl Live for Sexpr {
    fn lifetime(&self) -> LifetimeBorrow {
        self.lifetime.borrow_lifetime()
    }
}

impl Typed for Sexpr {
    #[inline]
    fn ty(&self) -> TypeRef {
        self.ty.borrow_ty()
    }
    #[inline]
    fn is_ty(&self) -> bool {
        match self.len() {
            0 => false,
            1 => self[0].is_ty(),
            _ => self.ty().is_universe(),
        }
    }
}

impl Value for Sexpr {
    #[inline]
    fn no_deps(&self) -> usize {
        self.len()
    }
    #[inline]
    fn get_dep(&self, ix: usize) -> &ValId {
        &self[ix]
    }
}

impl Deref for Sexpr {
    type Target = SexprArgs;
    fn deref(&self) -> &SexprArgs {
        &self.args
    }
}

impl Apply for Sexpr {}

impl Substitute for Sexpr {
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<Sexpr, Error> {
        let args: Result<_, _> = self
            .args
            .iter()
            .cloned()
            .map(|val| val.substitute(ctx))
            .collect();
        Sexpr::try_new(args?)
    }
}

impl SubstituteToValId for Sexpr {}

#[cfg(feature = "prettyprinter")]
mod prettyprint_impl {
    use super::*;
    use crate::prettyprinter::{tokens::*, PrettyPrint, PrettyPrinter};
    use std::fmt::{self, Display, Formatter};

    impl PrettyPrint for Sexpr {
        fn prettyprint<I: From<usize> + Display>(
            &self,
            printer: &mut PrettyPrinter<I>,
            fmt: &mut Formatter,
        ) -> Result<(), fmt::Error> {
            write!(fmt, "{}", SEXPR_OPEN)?;
            let mut first = true;
            for value in self.iter() {
                if !first {
                    write!(fmt, " ")?;
                }
                first = true;
                value.prettyprint(printer, fmt)?;
            }
            write!(fmt, "{}", SEXPR_CLOSE)
        }
    }
}
