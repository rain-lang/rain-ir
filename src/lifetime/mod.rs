/*!
The `rain` lifetime system
*/
use crate::value::{ValId, ValRef};
use either::Either;
use fxhash::FxBuildHasher;
use indexmap::IndexMap;
use itertools::{EitherOrBoth, Itertools};
use smallvec::SmallVec;
use std::iter::Copied;

/// A system of `rain` lifetimes
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LifetimeCtx {
    /// The compound lifetimes in this system
    lifetimes: IndexMap<Lenders, GroupData, FxBuildHasher>,
    /// The nodes in this system
    nodes: IndexMap<ValId, NodeData, FxBuildHasher>,
}

impl LifetimeCtx {
    /// Get the group at a given ID
    #[inline]
    pub fn group(&self, id: GroupId) -> Group {
        let (lenders, data) = self.lifetimes.get_index(id.0).unwrap();
        Group { lenders, data }
    }
    /// Get the node at a given ID
    #[inline]
    pub fn node(&self, id: NodeId) -> Node {
        let (value, data) = self.nodes.get_index(id.0).unwrap();
        Node {
            value: value.borrow_val(),
            data,
        }
    }
    /// Get the node or group at a given abstract ID
    #[inline]
    pub fn lifetime(&self, id: LifetimeId) -> Either<Node, Group> {
        match id.to_either() {
            Either::Left(id) => Either::Left(self.node(id)),
            Either::Right(id) => Either::Right(self.group(id)),
        }
    }
    /// Iterate over the borrowers of a given node
    #[inline]
    pub fn borrowers(&self, node: NodeId) -> Borrowers {
        self.node(node).borrowers(self)
    }
}

/// A node in a lifetime graph
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Node<'a> {
    /// The `ValRef` of this node
    pub value: ValRef<'a>,
    /// The data of this node
    pub data: &'a NodeData,
}

impl<'a> Node<'a> {
    /// Iterate over the borrowers of this node within a given context
    pub fn borrowers(self, ctx: &'a LifetimeCtx) -> Borrowers<'a> {
        Borrowers {
            abstract_borrowers: self.data.borrowers.iter(),
            lifetime_borrowers: [].iter(),
            ctx,
        }
    }
}

/// The size of a small vector of lifetime parameters
pub const SMALL_LIFETIME_PARAMS: usize = 2;

/// The size of a small vector of borrowers
const SMALL_BORROWERS: usize = 2;

/// The data associated with a node in a lifetime graph
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NodeData {
    /// The owner of this node
    ///
    /// TODO: field borrows
    owner: NodeOwnership,
    /// The lifetime-vector of this node
    lifetime: SmallVec<[LifetimeId; SMALL_LIFETIME_PARAMS]>,
    /// The nodes and lifetimes borrowing from this node
    borrowers: SmallVec<[LifetimeId; SMALL_BORROWERS]>,
}

/// The ownership status of a node in a lifetime graph
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum NodeOwnership {
    /// This node is completely owned by another node
    Owned(NodeId),
    /// This node is completely borrowed from a lifetime or another node
    Borrowed(LifetimeId),
    //TODO: field owners, etc.
}

/// An iterator over the borrowers of a node
#[derive(Debug, Clone)]
pub struct Borrowers<'a> {
    abstract_borrowers: std::slice::Iter<'a, LifetimeId>,
    lifetime_borrowers: std::slice::Iter<'a, NodeId>,
    ctx: &'a LifetimeCtx,
}

impl Iterator for Borrowers<'_> {
    type Item = NodeId;
    #[inline]
    fn next(&mut self) -> Option<NodeId> {
        loop {
            let next_from_curr = self.lifetime_borrowers.next();
            if let Some(next) = next_from_curr {
                return Some(*next);
            }
            let next_abstract = *self.abstract_borrowers.next()?;
            match next_abstract.to_either() {
                Either::Left(node) => return Some(node),
                Either::Right(lifetime) => {
                    self.lifetime_borrowers = self.ctx.group(lifetime).data.borrowers.iter()
                }
            }
        }
    }
}

impl NodeData {
    /// Tidy node data, deduplicating and sorting the borrower list, etc.
    pub fn tidy(&mut self) {
        self.borrowers.sort();
        self.borrowers.dedup();
        self.borrowers.shrink_to_fit();
    }
}

/// A group of objects borrowed from simultaneously, forming a `rain` lifetime
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Group<'a> {
    /// The lenders of this lifetime
    pub lenders: &'a Lenders,
    /// The data of this lifetime
    pub data: &'a GroupData,
}

/// The data associated with a lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct GroupData {
    /// The borrowers of this lifetime
    borrowers: Vec<NodeId>,
}

impl GroupData {
    /// Tidy lifetime data, deduplicating and sorting the borrower list, etc.
    pub fn tidy(&mut self) {
        self.borrowers.sort();
        self.borrowers.dedup();
        self.borrowers.shrink_to_fit();
    }
}

/// A lifetime ID
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct GroupId(usize);

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

/// A node ID
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct NodeId(usize);

/// An ID which is either a lifetime ID or a node ID
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct LifetimeId(usize);

impl LifetimeId {
    /// Check whether this `LifetimeId` is a node
    #[inline]
    pub fn is_node(self) -> bool {
        self.0 % 2 == 0
    }
    /// Check whether this `LifetimeId` is a lifetime
    #[inline]
    pub fn is_group(self) -> bool {
        self.0 % 2 == 1
    }
    #[inline]
    fn to_ix(self) -> usize {
        self.0 >> 2
    }
    /// Get this `LifetimeId` as either a `NodeId` or `GroupId`
    #[inline]
    pub fn to_either(self) -> Either<NodeId, GroupId> {
        if self.is_node() {
            Either::Left(NodeId(self.to_ix()))
        } else {
            Either::Right(GroupId(self.to_ix()))
        }
    }
    /// Try to get this `LifetimeId` as a node. Guaranteed to succeed if `is_node` returns `true`.
    #[inline]
    pub fn try_node(self) -> Option<NodeId> {
        if self.is_node() {
            Some(NodeId(self.to_ix()))
        } else {
            None
        }
    }
    /// Try to get this `LifetimeId` as a lifetime. Guaranteed to succeed if `is_lifetime` returns `true`.
    #[inline]
    pub fn try_group(self) -> Option<GroupId> {
        if self.is_group() {
            Some(GroupId(self.to_ix()))
        } else {
            None
        }
    }
    /// Check whether this `LifetimeId` is the static lifetime
    #[inline]
    pub fn is_static(self) -> bool {
        self.0 == usize::MAX
    }
}

impl From<NodeId> for LifetimeId {
    fn from(node: NodeId) -> LifetimeId {
        LifetimeId(node.0 << 1)
    }
}

impl From<GroupId> for LifetimeId {
    fn from(lifetime: GroupId) -> LifetimeId {
        // Wrapping shl + 1 because `Group::STATIC` is usize::MAX
        LifetimeId(lifetime.0.wrapping_shl(1) + 1)
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
}
