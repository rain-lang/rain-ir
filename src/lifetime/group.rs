/*!
Lifetime borrow-groups
*/

use super::*;
use erasable::Thin;
use slice_dst::SliceWithHeader;

lazy_static! {
    /// The global cache of sets of lifetimes, i.e. multigroups
    pub static ref MULTIGROUP_CACHE: DashCache<GSArc> = DashCache::new();
}

//TODO: proper eq, partialeq, hash, etc.

/// A non-empty group of values which is being borrowed from
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Group(Union2<Arc<ValueEnum>, Arc<MultiGroup>>);

/// An arc to a slice of groups
pub type GSArc = Arc<SliceWithHeader<(), Group>>;

/// A group of more than one value being borrowed from
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MultiGroup(Thin<GSArc>);
