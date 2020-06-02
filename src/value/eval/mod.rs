/*!
`rain` value evaluation.
*/

use super::{
    lifetime::{Lifetime, Live},
    typing::Typed,
    TypeId, ValId,
};

/// An evaluation error
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error {
    /// Attempting to apply a non-function
    NotAFunction,
}

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
    fn do_apply<'a>(&self, args: &'a [ValId], _inline: bool) -> Result<Application<'a>, Error> {
        if args.len() == 0 {
            Ok(Application::Complete(self.lifetime().clone_lifetime(), self.ty().clone_ty()))
        } else {
            Err(Error::NotAFunction)
        }
    }
}
