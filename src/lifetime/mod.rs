/*!
The `rain` lifetime system
*/
use crate::value::{Error, ValAddr, ValId, ValRef};
use fxhash::FxHashMap as HashMap;

/// A data structure tracking ownerships and borrowing within a single parametrized value
pub struct OwnerTable {

}