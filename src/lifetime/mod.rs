/*!
The `rain` lifetime system
*/
use crate::value::Error;
use crate::value::{ValId, ValRef};
use fxhash::FxBuildHasher;
use indexmap::{map::Entry, IndexMap};
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
    /// The current number of abstract identifiers
    abstract_ids: usize,
}

/// A node ID
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct NodeId(usize);

/// An abstract lifetime
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct AbstractId(pub usize);

/// An enum of possible ID kinds
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum IdEnum {
    /// A node ID
    Node(NodeId),
    /// An abstract ID
    Abstract(AbstractId),
    /// A group ID
    Group(GroupId),
}

/// An ID which is either a node ID, abstract ID or group ID
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

/// An enum of possible lifetime kinds
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum LifetimeEnum<'a> {
    /// A node
    Node(Node<'a>),
    /// An abstract lifetime
    Abstract(AbstractId),
    /// A group ID
    Group(Group<'a>),
}

/// The data associated with a node in a lifetime graph
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NodeData {
    /// The consumer of this node, if any
    ///
    /// TODO: field borrows
    consumer: Option<Owner>,
    /// The lifetime-vector of this node
    lifetime: LifetimeParams,
    /// The nodes and lifetimes borrowing from this node
    borrowers: SmallVec<[LifetimeId; NodeData::SMALL_BORROWERS]>,
}

/// The ownership status of a node in a lifetime graph
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Owner {
    /// This node is completely owned by another node
    Owned(NodeId),
    /// This node is completely borrowed from a lifetime or another node
    Borrowed(LifetimeId),
    //TODO: field owners, etc.
}

impl LifetimeCtx {
    /// Create a new, empty system of lifetimes
    #[inline]
    pub fn new(abstract_ids: usize) -> LifetimeCtx {
        LifetimeCtx {
            groups: IndexMap::default(),
            nodes: IndexMap::default(),
            abstract_ids,
        }
    }
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
    pub fn lifetime(&self, id: LifetimeId) -> LifetimeEnum {
        match id.to_enum() {
            IdEnum::Node(id) => LifetimeEnum::Node(self.node(id)),
            IdEnum::Abstract(id) => LifetimeEnum::Abstract(id),
            IdEnum::Group(id) => LifetimeEnum::Group(self.group(id)),
        }
    }
    /// Iterate over the borrowers of a given node
    #[inline]
    pub fn borrowers(&self, node: NodeId) -> Borrowers {
        self.node(node).borrowers(self)
    }
    /// Insert the given node into the table with the given lifetime parameters. Return an error if this value has already been inserted.
    ///
    /// Return an error on an incompatible consumer
    #[inline]
    pub fn force_insert(
        &mut self,
        node: ValId,
        consumer: Option<Owner>,
        lifetime: LifetimeParams,
    ) -> Result<NodeId, Error> {
        let ix = match self.nodes.entry(node) {
            Entry::Occupied(_) => {
                // Fix this...
                return Err(Error::AffineUsed);
            }
            Entry::Vacant(v) => {
                let data = NodeData {
                    lifetime,
                    consumer,
                    borrowers: SmallVec::new(),
                };
                let ix = v.index();
                v.insert(data);
                ix
            }
        };
        Ok(NodeId(ix))
    }
    /// Insert the given node into the table if it is not already present, with the given lifetime computation function and optional consumer.
    #[inline]
    pub fn insert<L>(
        &mut self,
        node: &ValId,
        consumer: Option<Owner>,
        mut compute_lifetime: L,
    ) -> Result<(NodeId, bool), Error>
    where
        L: FnMut(&LifetimeCtx, &ValId) -> LifetimeParams,
    {
        if let Some((ix, valid, node_data)) = self.nodes.get_full_mut(node) {
            debug_assert_eq!(valid, node);
            if node_data.consumer.is_some() && consumer.is_some() {
                //TODO: more specific...
                return Err(Error::AffineBranched);
            }
            if node_data.consumer.is_none() {
                node_data.consumer = consumer
            }
            return Ok((NodeId(ix), false));
        }
        let lifetime = compute_lifetime(self, node);
        Ok((
            self.force_insert(node.clone(), consumer, lifetime)
                .expect("No entry for this node exists in the node map"),
            true,
        ))
    }
}

impl Default for LifetimeCtx {
    #[inline]
    fn default() -> LifetimeCtx {
        LifetimeCtx::new(0)
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
            loop {
                let next_abstract = *self.borrowers.next()?;
                match next_abstract.to_enum() {
                    IdEnum::Node(node) => return Some(node),
                    IdEnum::Abstract(_) => {}
                    IdEnum::Group(group) => {
                        self.group_borrowers = self.ctx.group(group).data.borrowers().iter();
                        break;
                    }
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
    /// The static lifetime
    pub const STATIC: LifetimeId = LifetimeId((usize::MAX & !Self::BITMASK) + Self::GROUP_DISC);
    /// The discriminant for nodes
    const NODE_DISC: usize = 0;
    /// The discriminant for abstract nodes
    const ABSTRACT_DISC: usize = 1;
    /// The discriminant for groups
    const GROUP_DISC: usize = 2;
    /// The number of bits shifted
    const BITS_SHIFTED: u32 = 2;
    /// The bitmask used
    const BITMASK: usize = (1 << Self::BITS_SHIFTED) - 1;
    /// Check whether this `LifetimeId` is a node
    #[inline]
    pub fn is_node(self) -> bool {
        self.0 & Self::BITMASK == Self::NODE_DISC
    }
    /// Chech whether this `LifetimeId` is abstract
    #[inline]
    pub fn is_abstract(self) -> bool {
        self.0 & Self::BITMASK == Self::ABSTRACT_DISC
    }
    /// Check whether this `LifetimeId` is a lifetime
    #[inline]
    pub fn is_group(self) -> bool {
        self.0 % Self::BITMASK == Self::GROUP_DISC
    }
    #[inline]
    fn to_ix(self) -> usize {
        self.0 >> 2
    }
    /// Get this `LifetimeId` as an `IdEnum`
    #[inline]
    pub fn to_enum(self) -> IdEnum {
        match self.0 & Self::BITMASK {
            Self::NODE_DISC => IdEnum::Node(NodeId(self.to_ix())),
            Self::ABSTRACT_DISC => IdEnum::Abstract(AbstractId(self.to_ix())),
            Self::GROUP_DISC => IdEnum::Group(GroupId(self.to_ix())),
            discriminant => unreachable!("Invalid lifetime discriminant {}!", discriminant),
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
    /// Try to get this `LifetimeId` as an abstract ID. guaranteed to succeed if `is_abstract` returns `true`.
    #[inline]
    pub fn try_abstract(self) -> Option<AbstractId> {
        if self.is_abstract() {
            Some(AbstractId(self.to_ix()))
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
        self == Self::STATIC
    }
}

impl From<NodeId> for LifetimeId {
    fn from(node: NodeId) -> LifetimeId {
        LifetimeId((node.0 << LifetimeId::BITS_SHIFTED) + LifetimeId::NODE_DISC)
    }
}

impl From<AbstractId> for LifetimeId {
    fn from(node: AbstractId) -> LifetimeId {
        LifetimeId((node.0 << LifetimeId::BITS_SHIFTED) + LifetimeId::ABSTRACT_DISC)
    }
}

impl From<GroupId> for LifetimeId {
    fn from(lifetime: GroupId) -> LifetimeId {
        // Wrapping shl + 1 because `Group::STATIC` is usize::MAX
        LifetimeId((lifetime.0 << LifetimeId::BITS_SHIFTED) + LifetimeId::GROUP_DISC)
    }
}

impl From<NodeId> for IdEnum {
    fn from(node: NodeId) -> IdEnum {
        IdEnum::Node(node)
    }
}

impl From<AbstractId> for IdEnum {
    fn from(node: AbstractId) -> IdEnum {
        IdEnum::Abstract(node)
    }
}

impl From<GroupId> for IdEnum {
    fn from(lifetime: GroupId) -> IdEnum {
        IdEnum::Group(lifetime)
    }
}

impl From<IdEnum> for LifetimeId {
    fn from(id: IdEnum) -> LifetimeId {
        match id {
            IdEnum::Node(n) => n.into(),
            IdEnum::Abstract(a) => a.into(),
            IdEnum::Group(g) => g.into(),
        }
    }
}

impl From<LifetimeId> for IdEnum {
    fn from(lt: LifetimeId) -> IdEnum {
        lt.to_enum()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn lifetime_id_construction() {
        let sample_numbers = [0, 1, 2, 3, 6, 62, 4567354];
        for number in sample_numbers.iter().copied() {
            let node_id = NodeId(number);
            let abstract_id = AbstractId(number);
            let group_id = GroupId(number);
            let lt_node_id = LifetimeId::from(node_id);
            let lt_abstract_id = LifetimeId::from(abstract_id);
            let lt_group_id = LifetimeId::from(group_id);
            let en_node_id = IdEnum::from(node_id);
            let en_abstract_id = IdEnum::from(abstract_id);
            let en_group_id = IdEnum::from(group_id);
            assert_eq!(lt_node_id, LifetimeId::from(en_node_id));
            assert_eq!(IdEnum::from(lt_node_id), en_node_id);
            assert_eq!(lt_abstract_id, LifetimeId::from(en_abstract_id));
            assert_eq!(IdEnum::from(lt_abstract_id), en_abstract_id);
            assert_eq!(lt_group_id, LifetimeId::from(en_group_id));
            assert_eq!(IdEnum::from(lt_group_id), en_group_id);
        }
        assert!(LifetimeId::STATIC.is_group());
        assert_eq!(LifetimeId::STATIC.try_group(), Some(GroupId::STATIC));
        assert_eq!(LifetimeId::STATIC.try_node(), None);
        assert_eq!(LifetimeId::STATIC.try_abstract(), None);
    }
}
