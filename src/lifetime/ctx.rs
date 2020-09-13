/*!
A `rain` lifetime context.
*/
use super::*;

/// A `rain` lifetime context
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LifetimeCtx {
    /// The underlying ownership graph
    owners: OwnerGraph,
    /// The underlying lifetime graph
    lifetimes: LifetimeGraph,
}
