/*!
`rain` value evaluation.
*/

use super::{
    lifetime::{Lifetime, Live},
    typing::Typed,
    Error, TypeId, ValId,
};
mod ctx;
pub use ctx::EvalCtx;

/// The result of a *valid* application. An invalid application should return an error!
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Application<'a, V = ValId> {
    /// Stopped evaluation: not enough information without a change in parameters given
    Stop(Lifetime, TypeId),
    /// Complete evaluation: any more parameters will cause a failure
    Complete(Lifetime, TypeId),
    /// Incomplete information: may be enough information with more parameters
    Incomplete(Lifetime, TypeId),
    /// A successful evaluation to a value
    Success(&'a [ValId], V),
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
        impl<U: From<$T>> crate::value::eval::Substitute<U> for $T {
            /// Substitute this object in the given context: this is always a no-op
            fn substitute(
                &self,
                _ctx: &mut $crate::value::eval::EvalCtx,
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
        impl $crate::value::eval::Substitute<$crate::value::ValId> for $T
        where
            $T: Into<$crate::value::ValId>,
        {
            fn substitute(
                &self,
                ctx: &mut $crate::value::eval::EvalCtx,
            ) -> Result<$crate::value::ValId, $crate::value::Error> {
                let sub: $T = self.substitute(ctx)?;
                Ok(sub.into())
            }
        }
        impl $crate::value::eval::Substitute<$crate::value::ValueEnum> for $T
        where
            $T: Into<$crate::value::ValueEnum>,
        {
            fn substitute(
                &self,
                ctx: &mut $crate::value::eval::EvalCtx,
            ) -> Result<$crate::value::ValueEnum, $crate::value::Error> {
                let sub: $T = self.substitute(ctx)?;
                Ok(sub.into())
            }
        }
    };
}

/// An object which can be applied to a list of `rain` values
pub trait Apply: Typed + Live {
    /**
    Attempt to apply an object to a list of `rain` values, returning an `Application`.
    Currying, while not incorrect behaviour, is optional to implementors and hence not to be relied on.
    Use a loop to be sure!
    */
    #[inline]
    fn apply<'a>(&self, args: &'a [ValId]) -> Result<Application<'a>, Error> {
        self.do_apply(args, false)
    }
    /**
    Attempt to apply an object to a list of `rain` values, returning an `Application` and always inlining.
    Currying, while not incorrect behaviour, is optional to implementors and hence not to be relied on.
    Use a loop to be sure!
    */
    #[inline]
    fn inline<'a>(&self, args: &'a [ValId]) -> Result<Application<'a>, Error> {
        self.do_apply(args, true)
    }
    /**
    Attempt to apply an object to a list of `rain` values, returning an `Application`, and optionally inlining.

    Currying, while not incorrect behaviour, is optional to implementors and hence not to be relied on.
    Use a loop to be sure!
    */
    fn do_apply<'a>(&self, args: &'a [ValId], inline: bool) -> Result<Application<'a>, Error> {
        self.do_apply_in_ctx(args, inline, None)
    }
    /**
    Attempt to apply an object to a list of `rain` values within an (optional) evaluation context

    Currying, while not incorrect behaviour, is optional to implementors and hence not to be relied on.
    Use a loop to be sure!
    */
    fn do_apply_in_ctx<'a>(
        &self,
        args: &'a [ValId],
        _inline: bool,
        _ctx: Option<&mut EvalCtx>,
    ) -> Result<Application<'a>, Error> {
        if args.len() == 0 {
            Ok(Application::Complete(
                self.lifetime().clone_lifetime(),
                self.ty().clone_ty(),
            ))
        } else {
            Err(Error::NotAFunction)
        }
    }
}
