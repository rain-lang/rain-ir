/*!
`rain` value evaluation.
*/

use super::ValId;

/// An evaluation error
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error {
    /// Attempting to apply a non-function
    NotAFunction,
}

/// The result of a *valid* evaluation. An invalid evaluation should return an error!
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Application<'a, V = ValId> {
    /// Stop evaluation: not enough information without a change in parameters given
    Stop(&'a [ValId]),
    /// Incomplete information: may be enough information with more parameters
    Incomplete(&'a [ValId]),
    /// A successful evaluation to a value
    Success(&'a [ValId], V),
}

/// An object which can be applied to a list of `rain` values
pub trait Apply {
    /**
    Attempt to apply an object to a list of `rain` values, returning an `Application`.
    Currying, while not incorrect behaviour, is optional to implementors and hence not to be relied on.
    Use a loop to be sure!
    */
    #[inline]
    fn apply(&self, args: &[ValId]) -> Result<Application, Error> {
        self.do_apply(args, false)
    }
    /**
    Attempt to apply an object to a list of `rain` values, returning an `Application` and always inlining.
    Currying, while not incorrect behaviour, is optional to implementors and hence not to be relied on.
    Use a loop to be sure!
    */
    #[inline]
    fn inline(&self, args: &[ValId]) -> Result<Application, Error> {
        self.do_apply(args, true)
    }
    /**
    Attempt to apply an object to a list of `rain` values, returning an `Application`, and optionally inlining.
    Currying, while not incorrect behaviour, is optional to implementors and hence not to be relied on.
    Use a loop to be sure!
    */
    fn do_apply(&self, _args: &[ValId], _inline: bool) -> Result<Application, Error> {
        Err(Error::NotAFunction)
    }
}
