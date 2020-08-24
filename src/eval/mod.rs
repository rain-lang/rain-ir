/*!
`rain` value evaluation.
*/

use super::typing::{Type, Typed};
use crate::region::Regional;
use crate::value::{expr::Sexpr, Error, TypeId, ValId, Value};
mod ctx;
pub use ctx::EvalCtx;

/// The result of a *valid* application. An invalid application should return an error!
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Application<'a, V = ValId> {
    /// A symbolic evaluation to a type
    Symbolic(TypeId),
    /// A successful evaluation to a value
    Success(&'a [ValId], V),
}

impl<'a> Application<'a> {
    /// Convert any application into a successful application
    pub(crate) fn valid_to_success<V: Value + Clone>(
        self,
        value: &V,
        args: &[ValId],
    ) -> (&'a [ValId], ValId) {
        let ty = match self {
            Application::Symbolic(ty) => ty,
            Application::Success(rest, val) => return (rest, val),
        };
        let mut new_args = Vec::with_capacity(1 + args.len());
        new_args.push(value.clone().into_val());
        new_args.extend_from_slice(args);
        let region = ty.gcrs(new_args.iter()).unwrap().clone_region();
        (
            &[],
            Sexpr::new_unchecked(new_args.into_iter().collect(), region, ty).into_val(),
        )
    }
}

/// An object which can have its components substituted to yield another (of type `S`)
pub trait Substitute<S = Self> {
    /// Substitute this object in the given context
    fn substitute(&self, ctx: &mut EvalCtx) -> Result<S, Error>;
}

/// Implemented `Substitute` trivially for a type which is `Clone`
#[macro_export]
macro_rules! trivial_substitute {
    ($T:ty) => {
        impl<U: From<$T>> crate::eval::Substitute<U> for $T {
            /// Substitute this object in the given context: this is always a no-op
            fn substitute(
                &self,
                _ctx: &mut $crate::eval::EvalCtx,
            ) -> Result<U, $crate::value::Error> {
                Ok(self.clone().into())
            }
        }
    };
}

/// Implemented `Substitute<ValId>` for a type implementing `Substitute<Self>`
#[macro_export]
macro_rules! substitute_to_valid {
    ($T:ty) => {
        impl $crate::eval::Substitute<$crate::value::ValId> for $T
        where
            $T: Into<$crate::value::ValId>,
        {
            fn substitute(
                &self,
                ctx: &mut $crate::eval::EvalCtx,
            ) -> Result<$crate::value::ValId, $crate::value::Error> {
                let sub: $T = self.substitute(ctx)?;
                Ok(sub.into())
            }
        }
        impl $crate::eval::Substitute<$crate::value::ValueEnum> for $T
        where
            $T: Into<$crate::value::ValueEnum>,
        {
            fn substitute(
                &self,
                ctx: &mut $crate::eval::EvalCtx,
            ) -> Result<$crate::value::ValueEnum, $crate::value::Error> {
                let sub: $T = self.substitute(ctx)?;
                Ok(sub.into())
            }
        }
    };
}

/// An object which can be applied to a list of `rain` values
pub trait Apply: Typed {
    /**
    Attempt to apply an object to a list of `rain` values, returning an `Application` on success.
    Currying, while not incorrect behaviour, is optional to implementors and hence not to be relied on.
    Use a loop (or call `applied`) to be sure!
    */
    #[inline]
    fn apply<'a>(&self, args: &'a [ValId]) -> Result<Application<'a>, Error> {
        self.apply_in(args, &mut None)
    }
    /**
    Attempt to apply an object to a list of `rain` values, returning an `Application` on success.
    Automatically curries as necessary
    */
    #[inline]
    fn curried<'a>(&self, args: &'a [ValId]) -> Result<Application<'a>, Error> {
        self.curried_in(args, &mut None)
    }
    /**
    Attempt to apply an object to a list of `rain` values in a given context, returning an `Application` on success.
    Currying, while not incorrect behaviour, is optional to implementors and hence not to be relied on.
    Use a loop (or call `applied_in`) to be sure!
    */
    #[inline]
    fn apply_in<'a>(
        &self,
        args: &'a [ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        self.ty().apply_ty_in(args, ctx).map(Application::Symbolic)
    }
    /**
    Attempt to apply an object to a list of `rain` values in a context, returning an `Application` on success.
    Automatically curries as necessary
    */
    #[inline]
    fn curried_in<'a>(
        &self,
        args: &'a [ValId],
        ctx: &mut Option<EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        let applied = self.apply_in(args, ctx)?;
        let (mut rest, mut value) = match applied {
            Application::Success(rest, value) => (rest, value),
            app => return Ok(app),
        };
        while !rest.is_empty() {
            let applied = value.apply_in(rest, ctx)?;
            let (new_rest, new_value) = match applied {
                Application::Success(rest, value) => (rest, value),
                app => return Ok(app),
            };
            if value == new_value || new_rest == rest {
                break;
            }
            rest = new_rest;
            value = new_value
        }
        Ok(Application::Success(rest, value))
    }
}
