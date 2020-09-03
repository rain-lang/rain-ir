/*!
Lifetime borrow-groups
*/

use super::*;

//TODO: proper eq, partialeq, hash, etc.

/// A non-empty group of values which is being borrowed from
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Group(Union2<Arc<ValueEnum>, Arc<MultiGroup>>);

/// A group of more than one value being borrowed from
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MultiGroup {

}