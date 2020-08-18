/*!
The `rain` lifetime system
*/
use crate::value::{Error, ValAddr, ValId, ValRef};
use fxhash::FxHashMap as HashMap;

/// A table of backlinks, recording when values in a region are borrowed and consumed
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BackTable {
    /// A map of `ValId` addresses to associated backlinks
    backlinks: HashMap<ValAddr, Backlinks>,
}

/// A set of backlinks for a given `ValId`
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Backlinks {
    /// The consumer of this `ValId`, if any
    consumer: Option<Consumer>,
    /// The borrowers of this `ValId`, if any
    borrowers: Vec<ValId>,
}

impl Default for Backlinks {
    fn default() -> Backlinks {
        Backlinks::new()
    }
}

impl Backlinks {
    /// Create a new, empty set of backlinks. Does not allocate.
    #[inline]
    pub fn new() -> Backlinks {
        Self::with_capacity(0)
    }
    /// Create a new set of backlinks, with a given borrower capacity
    #[inline]
    pub fn with_capacity(borrow_cap: usize) -> Backlinks {
        Backlinks {
            consumer: None,
            borrowers: Vec::with_capacity(borrow_cap),
        }
    }
    /// Sort the borrowers of this `ValId`
    #[inline]
    pub fn sort_borrowers(&mut self) {
        self.borrowers.sort_by_key(ValId::as_addr);
        self.borrowers.dedup()
    }
    /// Register a consumer of this `ValId`
    ///
    /// Return an error on conflict
    #[inline]
    pub fn register_consumer(&mut self, consumer: Consumer) -> Result<(), Error> {
        if self.consumer.is_none() {
            //TODO: field resolution, etc.
            self.consumer = Some(consumer);
            Ok(())
        } else {
            Err(Error::AffineUsed)
        }
    }
    /// Register a borrower of this `ValId`
    #[inline]
    pub fn register_borrower(&mut self, borrower: ValId) {
        self.borrowers.push(borrower)
    }
    /// Get the borrowers of this `ValId`
    #[inline]
    pub fn borrowers(&self) -> &[ValId] {
        &self.borrowers[..]
    }
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
