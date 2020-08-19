/*!
The `rain` lifetime system
*/
use crate::value::{Error, ValAddr, ValId, ValRef};
use fxhash::FxHashMap as HashMap;

/// A node ID, which is either a `ValAddr` or an `AbstractNode`
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct NodeId(usize);

impl From<NodeId> for NodeIdEnum {
    fn from(node: NodeId) -> NodeIdEnum {
        if node & 0b1 != 0 {
            NodeIdEnum::AbstractNode(node.0)
        } else {
            NodeIdEnum::ValAddr(ValAddr(node.0))
        }
    }
}

impl From<ValAddr> for NodeId {
    #[inline]
    fn from(addr: ValAddr) -> NodeId {
        NodeId(addr.0)
    }
}

/// A node ID enum, which represents the two possibilities:
pub enum NodeIdEnum {
    /// A value node
    Value(ValAddr),
    /// An abstract node
    AbstractNode(usize),
}

/// A data structure tracking ownerships and borrowing within a single parametrized value
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BorrowGraph {
    /// The nodes making up this graph
    nodes: HashMap<ValAddr, Node>,
}

/// A node in the borrow-checking graph
///
/// Each one of these represents a value of an affine type, by virtue of either owning or borrowing a resource
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Node {
    /// The consumer of this node in the graph
    ///
    /// This can be a single owner, consuming an entire node, or a different owner for each field
    consumer: Consumer,
    /// The nodes which this node borrows
    borrows: Vec<ValId>,
}

/// A consumer for a node in the borrow-checking graph
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Consumer {
    /// A single owner consuming an entire node
    Owner(ValId),
    //TODO: rest
}
