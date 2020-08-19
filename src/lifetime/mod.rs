/*!
The `rain` lifetime system
*/
use crate::value::{Error, ValAddr, ValId, ValRef};
use fxhash::FxHashMap as HashMap;

/// A data structure tracking ownerships and borrowing within a single parametrized value
pub struct BorrowGraph {
    /// The nodes making up this graph
    nodes: HashMap<ValAddr, Node>,
}

/// A node in the borrow-checking graph
///
/// Each one of these represents a value of an affine type, by virtue of either owning or borrowing a resource
pub struct Node {
    /// The consumer of this node in the graph
    /// 
    /// This can be a single owner, consuming an entire node, or a different owner for each field
    consumer: Consumer
}

/// A consumer for a node in the borrow-checking graph
pub enum Consumer {
    /// A single owner consuming an entire node
    Owner(ValId),
    //TODO: rest
}