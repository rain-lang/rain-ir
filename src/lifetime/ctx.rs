/*!
A `rain` lifetime context.
*/
use super::*;
use fxhash::FxBuildHasher;
use hashbrown::HashMap;

/**
A `rain` lifetime context graph

Handles:
- Checking affine values are only used once
- Checking relevant values are only used once
- Enumerating out-of-region dependencies of a `rain` value
- Generating a lifetime-component for a pi type having a given `rain` value as result
- Checking that a borrow-compatible topological sort of the `rain`-graph is possible
- Inserting the necessary temporal edges to guarantee a topological sort including those edges will be borrow-compatible
*/
#[derive(Debug, Clone)]
pub struct LifetimeCtx {
    /// The values in this lifetime context
    values: HashMap<ValId, NodeData, FxBuildHasher>,
    /// The groups in this lifetime context
    groups: HashMap<GroupId, NodeData, FxBuildHasher>,
}

/// The ID of a group in a `rain` lifetime context graph
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct GroupId(pub usize);

/// The ID of a node in a `rain` lifetime context graph
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct NodeId(pub usize);

/// The data associated with a node in a `rain` lifetime graph
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct NodeData {
    /// The consumer of this node, if any
    consumer: Option<Consumer>,
    /// The temporal edges leading to this node, if any
    temporal: Vec<NodeId>,
}

/// The consumer of a node in a `rain` lifetime graph
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Consumer {
    /// This node is owned by the listed source node
    ///
    /// This implies it must happen *before* the listed source node, but this is already handled by the dependency graph.
    Owner(NodeId),
    /// This node is borrowed from the listed source lender
    ///
    /// This implies it must happen *after* the listed source node, but this is already handled by the dependency graph.
    /// More importantly, however, this also implies it must happen *before* the *owner* of the listed source node, if any.
    Lender(NodeId),
}
