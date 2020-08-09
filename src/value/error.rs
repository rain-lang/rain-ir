/// An evaluation error
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error {
    /// Attempting to apply a non-function
    NotAFunction,
    /// Type mismatch
    TypeMismatch,
    /// Mismatched borrow merger
    BorrowedMismatch,
    /// Mismatched borrow source
    BorrowingMismatch,
    /// Multiple usage of an affine resource
    AffineUsed,
    /// Indirect multiple usage of an affine resource
    AffineBranched,
    /// Try to "move" an affine parameter out of a borrow
    AffineMove,
    /// Borrow of a used affine resource
    BorrowUsed,
    /// Unused relevant type
    RelevantUnused,
    /// Lifetime error
    LifetimeError,
    /// Incomparable regions
    IncomparableRegions,
    /// Incomparable lifetimes
    IncomparableLifetimes,
    /// Invalid casting into a lifetime
    InvalidCastIntoLifetime,
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
    InvalidSubKind,
    /// An invalid parameter
    InvalidParam,
    /// An undefined parameter
    UndefParam,
}
