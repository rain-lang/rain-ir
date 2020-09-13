/*!
A `rain` ownership graph
*/
use super::*;

/// A `rain` ownership graph
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OwnerGraph {
    /// The branches in this ownership graph
    branch_graph: BranchGraph
}