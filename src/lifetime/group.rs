use super::*;

/// A lifetime ID
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct GroupId(pub(super) usize);

impl GroupId {
    /// The ID of the static lifetime
    pub const STATIC: GroupId = GroupId(usize::MAX);
}

impl Default for GroupId {
    #[inline]
    fn default() -> GroupId {
        GroupId::STATIC
    }
}

/// A group of objects borrowed from simultaneously, forming a `rain` lifetime
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Group<'a> {
    /// The lenders composing this group
    pub lenders: &'a Lenders,
    /// The data associated with this lender-group
    pub data: &'a GroupData,
}

impl<'a> Group<'a> {
    /// Get the borrowers of this group
    #[inline]
    pub fn borrowers(&self) -> &[NodeId] {
        self.data.borrowers()
    }
}

/// The data associated with a lender-group
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct GroupData {
    /// The borrowers of this lender-group
    borrowers: Vec<NodeId>,
}

impl GroupData {
    /// Tidy group data, deduplicating and sorting the borrower list, etc.
    #[inline]
    pub fn tidy(&mut self) {
        self.borrowers.sort();
        self.borrowers.dedup();
        self.borrowers.shrink_to_fit();
    }
    /// Get the borrowers of this group
    #[inline]
    pub fn borrowers(&self) -> &[NodeId] {
        &self.borrowers[..]
    }
}

/// A set of lenders, represented as a sorted list of `usize` indices
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Lenders(Box<[NodeId]>);

impl Lenders {
    /// Directly create a new set of lenders from a sorted and deduplicated list of lenders.
    ///
    /// It is a logic error if this list is not sorted and deduplicated!
    #[inline]
    pub fn new_unchecked(data: Box<[NodeId]>) -> Lenders {
        Lenders(data)
    }
    /// Create a new set of lenders from a list of lenders not known to be sorted or deduplicated
    #[inline]
    pub fn new(mut data: Vec<NodeId>) -> Lenders {
        data.sort_unstable();
        data.dedup();
        data.shrink_to_fit();
        Self::new_unchecked(data.into_boxed_slice())
    }
    /// Check whether a node ID is in a set of lenders
    #[inline]
    pub fn is_lender(&self, node: NodeId) -> bool {
        self.0.binary_search(&node).is_ok()
    }
    /// Iterate over this set of lenders in sorted order
    #[inline]
    pub fn iter(&self) -> Copied<std::slice::Iter<NodeId>> {
        self.0.iter().copied()
    }
    /// Take the union of two sets of lenders
    #[inline]
    pub fn union(&self, other: &Lenders) -> Lenders {
        Self::new_unchecked(
            self.iter()
                .merge(other.iter())
                .dedup()
                .collect_vec()
                .into_boxed_slice(),
        )
    }
    /// Take the intersection of two sets of lenders
    #[inline]
    pub fn intersect(&self, other: &Lenders) -> Lenders {
        Self::new_unchecked(
            self.iter()
                .merge_join_by(other.iter(), Ord::cmp)
                .filter_map(|eob| match eob {
                    EitherOrBoth::Both(node, _) => Some(node),
                    _ => None,
                })
                .dedup()
                .collect_vec()
                .into_boxed_slice(),
        )
    }
    /// Take the union of an iterator of lenders
    #[inline]
    pub fn unions<'a, L>(lenders: L) -> Lenders
    where
        L: IntoIterator<Item = &'a Lenders>,
    {
        Self::new_unchecked(
            lenders
                .into_iter()
                .kmerge()
                .dedup()
                .collect_vec()
                .into_boxed_slice(),
        )
    }
}

impl IntoIterator for Lenders {
    type Item = NodeId;
    type IntoIter = std::vec::IntoIter<NodeId>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_vec().into_iter()
    }
}

impl<'a> IntoIterator for &'a Lenders {
    type Item = NodeId;
    type IntoIter = Copied<std::slice::Iter<'a, NodeId>>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
