/*!
A `rain` lifetime context.
*/
use super::*;

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
    groups: HashMap<Group, NodeData, FxBuildHasher>,
}

impl LifetimeCtx {
    /// Mutably get the data associated with a given `ValId`, inserting it if necessary
    pub fn valid_data_or_insert(&mut self, val: &ValId) -> Option<&mut NodeData> {
        self.values
            .lookup_mut(val, || Some((val.clone(), NodeData::default())))
            .map(|(_, data)| data)
    }
    /// Mutably get the data associated with a given `NodeId` if it already exists
    pub fn node_data_mut(&mut self, id: NodeId) -> Option<&mut NodeData> {
        match id.disc() {
            NodeId::VALID_DISC => self.values.lookup_mut(&id, || None).map(|(_, data)| data),
            NodeId::GROUP_DISC => self.groups.lookup_mut(&id, || None).map(|(_, data)| data),
            _ => None,
        }
    }
    /// Set the owner of a value in this lifetime context
    ///
    /// Return an error if this value is already owned or borrowed
    pub fn set_owner(&mut self, owned: &ValId, owner: NodeId) -> Result<(), Error> {
        self.valid_data_or_insert(owned)
            .expect("Always works...")
            .set_owner(owner)
    }
}

/// The ID of a node in a `rain` lifetime context graph
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct NodeId(pub usize);

impl NodeId {
    /// Get the discriminant corresponding to a `ValAddr`
    pub const VALID_DISC: usize = 0b0;
    /// Get the discriminant corresponding to a `GroupAddr`
    pub const GROUP_DISC: usize = 0b1;
    /// Get the mask to remove discriminants
    pub const DISC_MASK: usize = 0b11;
    /// Get the node ID corresponding to a value ID
    #[inline(always)]
    pub fn valid(valid: &ValId) -> NodeId {
        valid.as_addr().into()
    }
    /// Get the discriminant of this ID
    #[inline(always)]
    pub fn disc(self) -> usize {
        self.0 & Self::DISC_MASK
    }
}

impl From<ValAddr> for NodeId {
    #[inline(always)]
    fn from(valaddr: ValAddr) -> NodeId {
        NodeId(valaddr.0 | NodeId::VALID_DISC)
    }
}

impl From<GroupAddr> for NodeId {
    #[inline(always)]
    fn from(group_addr: GroupAddr) -> NodeId {
        NodeId(group_addr.0 | NodeId::GROUP_DISC)
    }
}

impl HasAddr for NodeId {
    #[inline(always)]
    fn raw_addr(&self) -> usize {
        self.0 & !Self::DISC_MASK
    }
}

/// The data associated with a node in a `rain` lifetime graph
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct NodeData {
    /// The consumer of this node, if any
    consumer: Option<Consumer>,
    /// The temporal edges leading to this node, if any
    temporal: Vec<NodeId>,
}

impl NodeData {
    /// Attempt to set a consumer of this node, returning an error if it already has an incompatible consumer
    pub fn set_consumer(&mut self, consumer: Consumer) -> Result<(), Error> {
        if self.consumer.is_none() {
            self.consumer = Some(consumer);
            Ok(())
        } else {
            Err(Error::AffineBranched)
        }
    }
    /// Attempt to set an owner for this node, returning an error if it already has an incompatible consumer
    pub fn set_owner(&mut self, owner: NodeId) -> Result<(), Error> {
        self.set_consumer(Consumer::Owner(owner))
    }
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
