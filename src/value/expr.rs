/*!
`rain` expressions
*/
use super::{arr::ValArr, Error, NormalValue, TypeId, TypeRef, ValId, Value, ValueData, ValueEnum};
use crate::eval::{Application, Apply, EvalCtx, Substitute};
use crate::lifetime::{Lifetime, LifetimeBorrow, Live};
use crate::primitive::UNIT_TY;
use crate::typing::{Type, Typed};
use crate::{debug_from_display, lifetime_region, pretty_display, substitute_to_valid, valarr};
use crate::enum_convert;
use std::ops::Deref;

/// An S-expression
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Sexpr {
    /// The arguments of this S-expression
    args: ValArr,
    /// The (cached) lifetime of this S-expression
    lifetime: Lifetime,
    /// The (cached) type of this S-expression
    ty: TypeId,
}

debug_from_display!(Sexpr);
pretty_display!(Sexpr, "(...)");

enum_convert! {
    impl InjectionRef<ValueEnum> for Sexpr {}
    impl TryFrom<NormalValue> for Sexpr { as ValueEnum, }
    impl TryFromRef<NormalValue> for Sexpr { as ValueEnum, }
}

impl Sexpr {
    /// Attempt to create an S-expression from an owned argument list, evaluating as necessary.
    pub fn try_new(mut args: Vec<ValId>) -> Result<Sexpr, Error> {
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
                let mut new_args = Vec::with_capacity(args.len() + s.len());
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
        Ok(Sexpr {
            args: args.into(),
            lifetime,
            ty,
        })
    }
    /// Attempt to create an S-expression from an un-owned argument-list, evaluating as necessary
    pub fn eval(args: &[ValId]) -> Result<Sexpr, Error> {
        match args.len() {
            0 => Ok(Self::unit()),
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
                    let mut a = Vec::with_capacity(1 + args.len());
                    a.push(f);
                    a.clone_from_slice(args);
                    return Ok(Sexpr {
                        args: a.into(),
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
            args: ValArr::EMPTY,
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
            args: valarr![value],
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

impl ValueData for Sexpr {}

lifetime_region!(Sexpr);

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
    #[inline]
    fn into_enum(self) -> ValueEnum {
        ValueEnum::Sexpr(self)
    }
    #[inline]
    fn into_norm(self) -> NormalValue {
        self.into()
    }
}

impl Deref for Sexpr {
    type Target = ValArr;
    fn deref(&self) -> &ValArr {
        &self.args
    }
}

impl Apply for Sexpr {}

impl Substitute for Sexpr {
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<Sexpr, Error> {
        use std::cmp::Ordering::*;
        let args: Result<_, _> = self
            .args
            .iter()
            .cloned()
            .map(|val| val.substitute(ctx))
            .collect();
        let lifetime = ctx.evaluate_lt(&self.lifetime)?;
        let mut result = Sexpr::try_new(args?)?;
        match result.lifetime.partial_cmp(&lifetime) {
            None => return Err(Error::LifetimeError),
            Some(Greater) => result.lifetime = lifetime,
            _ => {}
        };
        Ok(result)
    }
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

substitute_to_valid!(Sexpr);

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
                first = false;
                value.prettyprint(printer, fmt)?;
            }
            write!(fmt, "{}", SEXPR_CLOSE)
        }
    }
}
