/*!
Lifetime borrow-groups
*/

use super::*;

lazy_static! {
    /// The global cache of sets of lifetimes, i.e. multigroups
    pub static ref MULTIGROUP_CACHE: DashCache<GSArc> = DashCache::new();
}

/// A non-empty group of values which is being borrowed from
#[derive(Debug, Clone, Eq)]
pub struct Group(Union2<Arc<NormalValue>, Thin<GSArc>>);

impl Group {
    /// Get the pointer to the underlying data of this group
    #[inline]
    pub fn as_ptr(&self) -> ErasedPtr {
        self.0.as_untagged_ptr()
    }
}

impl From<MultiGroup> for Group {
    #[inline]
    fn from(group: MultiGroup) -> Group {
        Group(UnionAlign::right(group.0))
    }
}

impl From<ValId> for Option<Group> {
    #[inline]
    fn from(value: ValId) -> Option<Group> {
        value.into_arc().map(|value| Group(UnionAlign::left(value)))
    }
}

impl PartialEq for Group {
    #[inline]
    fn eq(&self, other: &Group) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}

impl Hash for Group {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.as_ptr().hash(hasher)
    }
}

/// An arc to a slice of groups
pub type GSArc = Arc<SliceWithHeader<(), Group>>;

/// A group of more than one value being borrowed from
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MultiGroup(Thin<GSArc>);
