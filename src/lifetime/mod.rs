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

impl BackTable {
    /// Validate and sort this table's backlinks. Return an error if this is impossible
    pub fn validate(&mut self) -> Result<(), Error> {
        for backlink in self.backlinks.values_mut() {
            backlink.validate()?
        }
        Ok(())
    }
    /// Register a consumer for a given address
    /// 
    /// Note this implicitly assumes the address is affine, and hence can be consumed
    pub fn register_consumer(&mut self, addr: ValAddr, consumer: Consumer) -> Result<(), Error> {
        self.backlinks
            .entry(addr)
            .or_default()
            .register_consumer(consumer)
    }
    /// Register a borrow for a given address
    /// 
    /// Note this implicitly assumes the address is affine, and hence can be borrowed
    pub fn register_borrow(&mut self, addr: ValAddr, borrow: ValId) {
        self.backlinks
            .entry(addr)
            .or_default()
            .register_borrow(borrow)
    }
}

/// A set of backlinks for a given `ValId`
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Backlinks {
    /// The consumer of this `ValId`, if any
    consumer: Option<Consumer>,
    /// The borrowers of this `ValId`, if any
    borrows: Vec<ValId>,
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
            borrows: Vec::with_capacity(borrow_cap),
        }
    }
    /// Sort the borrowers of this `ValId`
    #[inline]
    pub fn sort_borrows(&mut self) {
        self.borrows.sort_by_key(ValId::as_addr);
        self.borrows.dedup()
    }
    /// Register a consumer of this `ValId`
    ///
    /// Return an error on conflict
    pub fn register_consumer(&mut self, consumer: Consumer) -> Result<(), Error> {
        if self.consumer.is_none() {
            //TODO: field resolution, etc.
            self.consumer = Some(consumer);
            Ok(())
        } else {
            Err(Error::AffineUsed)
        }
    }
    /// Register a borrow of this `ValId`
    #[inline]
    pub fn register_borrow(&mut self, borrower: ValId) {
        self.borrows.push(borrower)
    }
    /// Get the borrows of this `ValId`
    #[inline]
    pub fn borrows(&self) -> &[ValId] {
        &self.borrows[..]
    }
    /// Validate the consumer of this `ValId` does not conflict with it's borrows, sorting it's borrow array
    ///
    /// Return an error on failed validation
    pub fn validate(&mut self) -> Result<(), Error> {
        self.sort_borrows();
        self.validate_borrows()
    }
    /// Validate the borrows of this `ValId`.
    ///
    /// # Correctness
    /// This assumes the borrow array is sorted: if it is not, calling this function is a logic error.
    /// Note this does *not* necessarily assume the borrow array is de-duplicated, though it generally
    /// should be.
    pub fn validate_borrows(&self) -> Result<(), Error> {
        if let Some(consumer) = &self.consumer {
            consumer.validate_borrows(self.borrows())
        } else {
            Ok(())
        }
    }
}

/// A consumer for a `ValId`
///
/// TODO: use an `ArcUnion` for more efficient storage?
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Consumer(ConsumerEnum);

impl Consumer {
    /// Borrow the consumer for this `ValId`
    #[inline]
    pub fn borrow_consumer(&self) -> ConsumerRef {
        self.0.borrow_consumer()
    }
    /// Get the consumer for this `ValId`
    #[inline]
    pub fn into_consumer(self) -> ConsumerEnum {
        self.0
    }
    /// Validate a sorted array of borrows is compatible with this consumer
    ///
    /// # Correctness
    /// This assumes the borrow array is sorted: if it is not, calling this function is a logic error.
    /// Note this does *not* necessarily assume the borrow array is de-duplicated, though it generally
    /// should be.
    #[inline]
    pub fn validate_borrows(&self, borrows: &[ValId]) -> Result<(), Error> {
        self.0.validate_borrows(borrows)
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
    /// Validate a sorted array of borrows is compatible with this consumer
    ///
    /// # Correctness
    /// This assumes the borrow array is sorted: if it is not, calling this function is a logic error.
    /// Note this does *not* necessarily assume the borrow array is de-duplicated, though it generally
    /// should be.
    #[inline]
    pub fn validate_borrows(&self, borrows: &[ValId]) -> Result<(), Error> {
        match self {
            ConsumerRef::Owned(v) => ConsumerEnum::validate_owned_consumer(v.as_valid(), borrows),
        }
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
    /// Validate an owned consumer
    #[inline]
    pub fn validate_owned_consumer(owned: &ValId, borrows: &[ValId]) -> Result<(), Error> {
        if borrows
            .binary_search_by_key(&owned.as_addr(), ValId::as_addr)
            .is_ok()
        {
            Err(Error::BorrowUsed)
        } else {
            Ok(())
        }
    }
    /// Validate a sorted array of borrows is compatible with this consumer
    ///
    /// # Correctness
    /// This assumes the borrow array is sorted: if it is not, calling this function is a logic error.
    /// Note this does *not* necessarily assume the borrow array is de-duplicated, though it generally
    /// should be.
    #[inline]
    pub fn validate_borrows(&self, borrows: &[ValId]) -> Result<(), Error> {
        match self {
            ConsumerEnum::Owned(v) => ConsumerEnum::validate_owned_consumer(v, borrows),
        }
    }
}
