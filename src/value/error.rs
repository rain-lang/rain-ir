/// An evaluation error
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error {
    /// Attempting to apply a non-function
    NotAFunction,
    /// Type mismatch
    TypeMismatch,
    /// Lifetime error
    LifetimeError,
    /// Incomparable regions
    IncomparableRegions,
    /// Evaluation error
    EvalError,
    /// Tuple length mismatch
    TupleLengthMismatch,
    /// Empty sexpr application
    EmptySexprApp,
    /// No inlining violation
    NoInlineError,
    /// A value is no longer a type after substitution
    NotATypeError,
    /// Too many arguments for a (non-curried!) function
    /// (or sometimes an object which *may* be a function)
    TooManyArgs,
    /// A pattern match failure
    MatchFailure,
    /// An incomplete match statement
    IncompleteMatch,
    /// An invalid substitution kind
    InvalidSubKind
}
