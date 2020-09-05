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

/// The address of a non-empty group of values
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct GroupAddr(pub usize);

impl From<ValAddr> for GroupAddr {
    #[inline(always)]
    fn from(val: ValAddr) -> GroupAddr {
        GroupAddr(val.0)
    }
}

impl PartialEq<ValAddr> for GroupAddr {
    #[inline(always)]
    fn eq(&self, other: &ValAddr) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<GroupAddr> for ValAddr {
    #[inline(always)]
    fn eq(&self, other: &GroupAddr) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd<ValAddr> for GroupAddr {
    #[inline(always)]
    fn partial_cmp(&self, other: &ValAddr) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl PartialOrd<GroupAddr> for ValAddr {
    #[inline(always)]
    fn partial_cmp(&self, other: &GroupAddr) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Group {
    /// Get the pointer to the underlying data of this group
    #[inline]
    pub fn as_ptr(&self) -> ErasedPtr {
        self.0.as_untagged_ptr()
    }
    /// Get the address of the underlying data of this group
    #[inline]
    pub fn addr(&self) -> GroupAddr {
        let unerased = unsafe { NonNull::<()>::unerase(self.0.as_untagged_ptr()) };
        GroupAddr(unerased.as_ptr() as usize)
    }
}

impl From<MultiGroup> for Group {
    #[inline]
    fn from(group: MultiGroup) -> Group {
        Group(UnionAlign::right(group.into()))
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

impl Drop for Group {
    #[inline]
    fn drop(&mut self) {
        self.0.with_a(|val| VALUE_CACHE.try_gc_global(val));
        self.0
            .with_b(|mgt| Thin::with(mgt, |mg| MULTIGROUP_CACHE.try_gc_global(mg)));
    }
}

/// An arc to a slice of groups
pub type GSArc = Arc<SliceWithHeader<(), Group>>;

/// A group of more than one value being borrowed from
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct MultiGroup(Thin<GSArc>);

impl Deref for MultiGroup {
    type Target = [Group];
    #[inline]
    fn deref(&self) -> &[Group] {
        &self.0.slice
    }
}

impl From<MultiGroup> for Thin<GSArc> {
    #[inline]
    fn from(mg: MultiGroup) -> Thin<GSArc> {
        unsafe { std::mem::transmute(mg) }
    }
}

impl Drop for MultiGroup {
    #[inline]
    fn drop(&mut self) {
        Thin::with(&self.0, |mg| MULTIGROUP_CACHE.try_gc_global(mg));
    }
}
