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
use std::ops::Deref;

mod group;
pub use group::*;
mod params;
pub use params::*;

/// A system of `rain` lifetimes
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LifetimeCtx {
    /// The compound lifetimes in this system
    groups: IndexMap<Lenders, GroupData, FxBuildHasher>,
    /// The nodes in this system
    nodes: IndexMap<ValId, NodeData, FxBuildHasher>,
}

/// A node ID
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct NodeId(usize);

/// An ID which is either a node ID or a group ID
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct LifetimeId(usize);

/// A node in a lifetime graph
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Node<'a> {
    /// The `ValRef` of this node
    pub value: ValRef<'a>,
    /// The data of this node
    pub data: &'a NodeData,
}

/// The data associated with a node in a lifetime graph
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NodeData {
    /// The owner of this node
    ///
    /// TODO: field borrows
    owner: NodeOwnership,
    /// The lifetime-vector of this node
    lifetime: LifetimeParams,
    /// The nodes and lifetimes borrowing from this node
    borrowers: SmallVec<[LifetimeId; NodeData::SMALL_BORROWERS]>,
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

impl LifetimeCtx {
    /// Get the group at a given ID
    #[inline]
    pub fn group(&self, id: GroupId) -> Group {
        let (lenders, data) = self.groups.get_index(id.0).unwrap();
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

impl<'a> Node<'a> {
    /// Iterate over the borrowers of this node within a given context
    pub fn borrowers(self, ctx: &'a LifetimeCtx) -> Borrowers<'a> {
        Borrowers {
            borrowers: self.data.borrowers.iter(),
            group_borrowers: [].iter(),
            ctx,
        }
    }
}

/// An iterator over the borrowers of a node
#[derive(Debug, Clone)]
pub struct Borrowers<'a> {
    borrowers: std::slice::Iter<'a, LifetimeId>,
    group_borrowers: std::slice::Iter<'a, NodeId>,
    ctx: &'a LifetimeCtx,
}

impl Iterator for Borrowers<'_> {
    type Item = NodeId;
    #[inline]
    fn next(&mut self) -> Option<NodeId> {
        loop {
            let next_from_curr = self.group_borrowers.next();
            if let Some(next) = next_from_curr {
                return Some(*next);
            }
            let next_abstract = *self.borrowers.next()?;
            match next_abstract.to_either() {
                Either::Left(node) => return Some(node),
                Either::Right(group) => {
                    self.group_borrowers = self.ctx.group(group).data.borrowers().iter()
                }
            }
        }
    }
}

impl NodeData {
    /// The size of a small vector of borrowers
    const SMALL_BORROWERS: usize = 2;
    /// Tidy node data, deduplicating and sorting the borrower list, etc.
    pub fn tidy(&mut self) {
        self.borrowers.sort();
        self.borrowers.dedup();
        self.borrowers.shrink_to_fit();
    }
}

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
