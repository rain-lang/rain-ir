/*!
The `rain` lifetime system
*/
use crate::value::{Error, ValAddr, ValId, ValRef};
use fxhash::FxHashMap as HashMap;

/// A data structure tracking ownerships and borrowing within a single parametrized value
pub struct OwnerTable {

}

/// A node in the ownership graph
/// 
/// Each one of these represents a value of an affine type, by virtue of either owning or borrowing a resource
pub struct Node {

}