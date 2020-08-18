/*!
The `rain` lifetime system
*/
use crate::value::{ValId, ValRef};
use fxhash::FxHashMap as HashMap;

/// A table of backlinks, recording when values in a region are borrowed and consumed
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BackTable {
    /// A map of `ValId` addresses to associated backlinks
    backlinks: HashMap<usize, Backlinks>,
}

/// A set of backlinks for a given `ValId`
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Backlinks {
    /// The consumer of this `ValId`, if any
    consumer: Option<Consumer>,
    /// The borrowers of this `ValId`, if any
    borrowers: Vec<ValId>,
}

/// A consumer for a `ValId`
///
/// TODO: use an `ArcUnion` for more efficient storage?
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Consumer(ConsumerEnum);

impl Consumer {
    /// Borrow the consumer for this `ValId`
    pub fn borrow_consumer(&self) -> ConsumerRef {
        self.0.borrow_consumer()
    }
    /// Get the consumer for this `ValId`
    pub fn into_consumer(self) -> ConsumerEnum {
        self.0
    }
}

/// A borrowed consumer for a `ValId`
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ConsumerRef<'a> {
    /// A single object completely consuming a value
    Owned(ValRef<'a>),
}

impl ConsumerRef<'_> {
    /// Clone this consumer as a `ConsumerEnum`
    #[inline]
    pub fn clone_enum(&self) -> ConsumerEnum {
        match self {
            ConsumerRef::Owned(o) => ConsumerEnum::Owned(o.clone_val()),
        }
    }
    /// Clone this consumer as a `Consumer`
    #[inline]
    pub fn clone_consumer(&self) -> Consumer {
        Consumer(self.clone_enum())
    }
}

/// An owned consumer
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ConsumerEnum {
    /// A single object completely consuming a value
    Owned(ValId),
}

impl ConsumerEnum {
    /// Borrow this consumer
    #[inline]
    pub fn borrow_consumer(&self) -> ConsumerRef {
        match self {
            ConsumerEnum::Owned(o) => ConsumerRef::Owned(o.borrow_val()),
        }
    }
}
