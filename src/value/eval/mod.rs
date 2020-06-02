/*!
`rain` value evaluation.
*/

use super::{ValId, TypeRef, typing::Typed};

/// An evaluation error
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error {
    /// Attempting to apply a non-function
    NotAFunction,
}

/// The result of a *valid* application. An invalid application should return an error!
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Application<V = ValId> {
    /// Stopped evaluation: not enough information without a change in parameters given
    Stop,
    /// Complete evaluation: any more parameters will cause a failure
    Complete,
    /// Incomplete information: may be enough information with more parameters
    Incomplete,
    /// A successful evaluation to a value
    Success(V),
}

/// The result of a valid application
pub type AppRes<'a> = (&'a [ValId], TypeRef<'a>, Application);

/// An object which can be applied to a list of `rain` values
pub trait Apply: Typed {
    /**
    Attempt to apply an object to a list of `rain` values, returning an `Application`.
    Currying, while not incorrect behaviour, is optional to implementors and hence not to be relied on.
    Use a loop to be sure!
    */
    #[inline]
    fn apply<'a>(&'a self, args: &'a [ValId]) -> Result<AppRes<'a>, Error> {
        self.do_apply(args, false)
    }
    /**
    Attempt to apply an object to a list of `rain` values, returning an `Application` and always inlining.
    Currying, while not incorrect behaviour, is optional to implementors and hence not to be relied on.
    Use a loop to be sure!
    */
    #[inline]
    fn inline<'a>(&'a self, args: &'a [ValId]) -> Result<AppRes<'a>, Error> {
        self.do_apply(args, true)
    }
    /**
    Attempt to apply an object to a list of `rain` values, returning an `Application`, and optionally inlining.
    Currying, while not incorrect behaviour, is optional to implementors and hence not to be relied on.
    Use a loop to be sure!
    */
    fn do_apply<'a>(&'a self, args: &'a [ValId], _inline: bool) -> Result<AppRes<'a>, Error> {
        if args.len() == 0 {
            Ok((args, self.ty(), Application::Complete))
        } else {
            Err(Error::NotAFunction)
        }
    }
}
