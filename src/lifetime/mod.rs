/*!
The `rain` lifetime system
*/
use crate::value::{ValId, ValRef};
use either::Either;
use fxhash::FxBuildHasher;
use indexmap::IndexMap;
use itertools::{EitherOrBoth, Itertools};
use std::iter::Copied;

/// A system of `rain` lifetimes
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LifetimeCtx {
    /// The compound lifetimes in this system
    lifetimes: IndexMap<Lenders, LifetimeData, FxBuildHasher>,
    /// The nodes in this system
    nodes: IndexMap<ValId, NodeData, FxBuildHasher>,
}

impl LifetimeCtx {
    /// Get the lifetime at a given ID
    #[inline]
    pub fn lifetime(&self, id: LifetimeId) -> Lifetime {
        let (lenders, data) = self.lifetimes.get_index(id.0).unwrap();
        Lifetime { lenders, data }
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
    /// Get the node or lifetime at a given abstract ID
    #[inline]
    pub fn object(&self, id: AbstractId) -> Either<Node, Lifetime> {
        match id.to_either() {
            Either::Left(id) => Either::Left(self.node(id)),
            Either::Right(id) => Either::Right(self.lifetime(id)),
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
    value: ValRef<'a>,
    /// The data of this node
    data: &'a NodeData,
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

/// The data associated with a node in a lifetime graph
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NodeData {
    /// The owner of this node
    ///
    /// TODO: field borrows
    owner: NodeId,
    /// The lifetime of this node
    ///
    /// TODO: field lifetimes?
    lifetime: LifetimeId,
    /// The nodes and lifetimes borrowing from this node
    borrowers: Vec<AbstractId>,
}

/// An iterator over the borrowers of a node
#[derive(Debug, Clone)]
pub struct Borrowers<'a> {
    abstract_borrowers: std::slice::Iter<'a, AbstractId>,
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
                    self.lifetime_borrowers = self.ctx.lifetime(lifetime).data.borrowers.iter()
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

/// A `rain` lifetime
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Lifetime<'a> {
    /// The lenders of this lifetime
    lenders: &'a Lenders,
    /// The data of this lifetime
    data: &'a LifetimeData,
}

/// The data associated with a lifetime
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LifetimeData {
    /// The borrowers of this lifetime
    borrowers: Vec<NodeId>,
}

impl LifetimeData {
    /// Tidy lifetime data, deduplicating and sorting the borrower list, etc.
    pub fn tidy(&mut self) {
        self.borrowers.sort();
        self.borrowers.dedup();
        self.borrowers.shrink_to_fit();
    }
}

/// A lifetime ID
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct LifetimeId(usize);

impl LifetimeId {
    /// The ID of the static lifetime
    pub const STATIC: LifetimeId = LifetimeId(usize::MAX);
}

impl Default for LifetimeId {
    #[inline]
    fn default() -> LifetimeId {
        LifetimeId::STATIC
    }
}

/// A node ID
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct NodeId(usize);

/// An ID which is either a lifetime ID or a node ID
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct AbstractId(usize);

impl AbstractId {
    /// Check whether this `AbstractId` is a node
    #[inline]
    pub fn is_node(self) -> bool {
        self.0 % 2 == 0
    }
    /// Check whether this `AbstractId` is a lifetime
    #[inline]
    pub fn is_lifetime(self) -> bool {
        self.0 % 2 == 1
    }
    #[inline]
    fn to_ix(self) -> usize {
        self.0 >> 2
    }
    /// Get this `AbstractId` as either a `NodeId` or `LifetimeId`
    #[inline]
    pub fn to_either(self) -> Either<NodeId, LifetimeId> {
        if self.is_node() {
            Either::Left(NodeId(self.to_ix()))
        } else {
            Either::Right(LifetimeId(self.to_ix()))
        }
    }
    /// Try to get this `AbstractId` as a node. Guaranteed to succeed if `is_node` returns `true`.
    #[inline]
    pub fn try_node(self) -> Option<NodeId> {
        if self.is_node() {
            Some(NodeId(self.to_ix()))
        } else {
            None
        }
    }
    /// Try to get this `AbstractId` as a lifetime. Guaranteed to succeed if `is_lifetime` returns `true`.
    #[inline]
    pub fn try_lifetime(self) -> Option<LifetimeId> {
        if self.is_lifetime() {
            Some(LifetimeId(self.to_ix()))
        } else {
            None
        }
    }
    /// Check whether this `AbstractId` is the static lifetime
    #[inline]
    pub fn is_static(self) -> bool {
        self.0 == usize::MAX
    }
}

impl From<NodeId> for AbstractId {
    fn from(node: NodeId) -> AbstractId {
        AbstractId(node.0 << 1)
    }
}

impl From<LifetimeId> for AbstractId {
    fn from(lifetime: LifetimeId) -> AbstractId {
        // Wrapping shl + 1 because `Lifetime::STATIC` is usize::MAX
        AbstractId(lifetime.0.wrapping_shl(1) + 1)
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
