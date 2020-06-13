/*!
Reference-counted, hash-consed, typed arrays of values
*/

use super::predicate::Is;
use super::{NormalValue, ValId, Value, VarId};
use crate::typing::TypeValue;
use crate::util::cache::{
    arr::{BagMarker, CachedArr, EmptyPredicate, SetMarker, Sorted, Uniq},
    Cache, Caches,
};
use itertools::Itertools;
use lazy_static::lazy_static;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::{Deref, Index};
use triomphe::{Arc, HeaderSlice, HeaderWithLength, ThinArc};

lazy_static! {
    /// A cache for arrays of values
    pub static ref ARRAY_CACHE: Cache<[ValId], CachedArr<ValId>> = Cache::default();
}

#[macro_export]
/// A macro to create a variable array
macro_rules! vararr {
    () => { $crate::value::arr::VarArr::EMPTY_SELF };
    ($elem:expr; $n:expr) => {{
        let v: Vec<VarId<_>> = vec![$elem; $n];
        v.into_iter().collect()
    }};
    ($($x:expr),+ $(,)?) => {{
        let v: Vec<VarId<_>> = vec![$($x,)+];
        v.into_iter().collect()
    }};
}

/// A reference-counted, hash-consed, typed array of values
#[derive(Debug, Eq, Hash)]
#[repr(transparent)]
pub struct ValArr<A = (), P = ()> {
    arr: CachedArr<ValId, A>,
    variant: std::marker::PhantomData<P>,
}

/// A reference-counted, hash-consed, typed array of values guaranteed to be a given variant.
pub type VarArr<V> = ValArr<(), Is<V>>;

impl<A, P> Clone for ValArr<A, P> {
    fn clone(&self) -> ValArr<A, P> {
        ValArr {
            arr: self.arr.clone(),
            variant: self.variant,
        }
    }
}

impl<A: EmptyPredicate, P> Default for ValArr<A, P> {
    /// Get an empty `VarArr`
    fn default() -> ValArr<A, P> {
        ValArr::EMPTY
    }
}

impl<A, B, P, Q> PartialEq<ValArr<A, P>> for ValArr<B, Q> {
    fn eq(&self, other: &ValArr<A, P>) -> bool {
        self.arr == other.arr
    }
}

/// The unique empty array of `ValId`s. Temporary hack...
static UNIQUE_EMPTY_ARRAY: [ValId; 0] = [];

impl<A: EmptyPredicate, P> ValArr<A, P> {
    /// This type as an empty array, for use in `const` contexts
    pub const EMPTY: ValArr<A, P> = ValArr {
        arr: CachedArr::EMPTY,
        variant: std::marker::PhantomData,
    };
}

impl<A, P> ValArr<A, P> {
    /// Get the length of this array
    #[inline]
    pub fn len(&self) -> usize {
        self.arr.len()
    }
    /// Check whether this array is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.arr.is_empty()
    }
    /// Get this array as an array of ValIds
    #[inline]
    pub fn as_valarr(&self) -> &ValArr {
        self.coerce_ref()
    }
    /// Forget any additional value information, yielding just a container of raw `ValId`s
    pub fn as_vals(&self) -> &ValArr<A, ()> {
        self.coerce_ref()
    }
    /// Forget any additional array information, yielding just a raw `ValArr`
    pub fn as_arr(&self) -> &ValArr<(), P> {
        self.coerce_ref()
    }
    /// Get this array as a slice of ValIds
    #[inline]
    pub fn as_slice(&self) -> &[ValId] {
        self.arr.as_slice()
    }
    /// Get this array as a pointer to it's first element, or null for an empty array
    #[inline]
    pub fn as_ptr(&self) -> *const ValId {
        self.arr.as_ptr()
    }
    /// Check if this array is address-sorted
    #[inline]
    pub fn is_sorted(&self) -> bool {
        self.arr.is_sorted()
    }
    /// Check that this array is address-sorted in a strictly increasing fashion
    #[inline]
    pub fn is_set(&self) -> bool {
        self.arr.is_set()
    }
    /// If this array is address-sorted, return it as a sorted array. If not, fail.
    #[inline]
    pub fn try_as_sorted(&self) -> Result<&ValBag<P>, &ValArr<A, P>> {
        if self.is_sorted() {
            Ok(self.coerce_ref())
        } else {
            Err(self)
        }
    }
    /// If this array is strictly address-sorted, return it as a set. If not, fail.
    #[inline]
    pub fn try_as_set(&self) -> Result<&ValSet<P>, &ValArr<A, P>> {
        if self.is_set() {
            Ok(self.coerce_ref())
        } else {
            Err(self)
        }
    }
    /// If this array is address-sorted, return it as a sorted array. If not, fail.
    #[inline]
    pub fn try_into_sorted(self) -> Result<ValBag<P>, ValArr<A, P>> {
        if self.is_sorted() {
            Ok(self.coerce())
        } else {
            Err(self)
        }
    }
    /// If this array is strictly address-sorted, return it as a set. If not, fail.
    #[inline]
    pub fn try_into_set(self) -> Result<ValSet<P>, ValArr<A, P>> {
        if self.is_set() {
            Ok(self.coerce())
        } else {
            Err(self)
        }
    }
    /// Clone this array as an array of ValIds
    #[inline]
    pub fn clone_vals(&self) -> ValArr {
        ValArr {
            arr: self.arr.clone().coerce(),
            variant: std::marker::PhantomData,
        }
    }
    /// Deduplicate a `CachedArr` to an array of `ValId`s, and assert this array is of the desired type
    #[inline]
    fn dedup(ava: CachedArr<ValId>) -> ValArr<A> {

    }
    /// Iterate over the `ValId`s in this container
    #[inline]
    fn iter_vals(&self) -> std::slice::Iter<ValId> {
        if let Some(arr) = &self.arr {
            arr.iter()
        } else {
            UNIQUE_EMPTY_ARRAY.iter()
        }
    }
    /// Address-sort this container *without* deduplicating
    #[inline]
    pub fn sorted(&self) -> ValBag<P> {
        // Special case (TODO: consider performance implications...)
        if self.is_sorted() {
            return self.clone().coerce();
        }
        ValBag::assert_new(self.iter_vals().sorted_by_key(|v| v.as_ptr()).cloned())
    }
    /// Address-sort this container *with* deduplication, yielding a `VarSet`
    #[inline]
    pub fn set(&self) -> ValSet<P> {
        let mut source: Vec<_> = self.iter_vals().sorted_by_key(|v| v.as_ptr()).collect();
        source.dedup();
        ValSet::assert_new(source.into_iter().cloned())
    }
    /// Coerce this container into a set of `VarId<V>`, asserting the predicate holds *for each container element*!
    #[inline]
    fn coerce<B, Q>(self) -> ValArr<B, Q> {
        ValArr {
            arr: self.arr.coerce(),
            variant: std::marker::PhantomData,
        }
    }
    /// Coerce a reference to this container into a set of `VarId<V>`, asserting the predicate holds *for each container element*!
    #[inline]
    fn coerce_ref<B, Q>(&self) -> &ValArr<B, Q> {
        unsafe { std::mem::transmute(self) }
    }
    /// Coerce this container into a slice of `VarId<V>`, asserting that the predicate holds *for each container element*!
    /// While this method can be called incorrectly, it is *safe* as regardless of `V`, all `VarId<V>` are guaranteed to have
    /// the same representation.
    #[inline]
    fn coerce_slice<U>(&self) -> &[ValId<U>] {
        unsafe { std::mem::transmute(self.arr.as_slice()) }
    }
}

/// An array of types
pub type TyArr = VarArr<TypeValue>;

/// A bag (implemented as a sorted array) of `rain` values
pub type VarBag<V> = ValArr<Sorted, Is<V>>;

/// A set (implemented as a sorted, unique array) of `rain` values
pub type VarSet<V> = ValArr<Uniq, Is<V>>;

impl ValBag {
    /// Check whether an item is in this bag. If it is, return a reference.
    /// This impl is to avoid recompilation for every `ValArr<A, P>`, `ValId<Q>` pair.
    pub fn contains_impl(&self, item: *const NormalValue) -> Option<&ValId> {
        self.as_slice()
            .binary_search_by_key(&item, ValId::as_ptr)
            .ok()
            .map(|ix| &self.as_slice()[ix])
    }
    /// Merge two bags
    /// This impl is to avoid recompilation for every `ValArr<A, P>`, `ValArr<B, Q>` pair.
    pub fn merge_impl(&self, rhs: &ValBag) -> ValBag {
        // Edge cases
        if rhs.is_empty() {
            return self.clone();
        } else if self.is_empty() {
            return rhs.clone();
        }
        let union: Vec<_> = self
            .iter_vals()
            .merge_by(rhs.iter_vals(), |l, r| l.as_ptr() <= r.as_ptr())
            .cloned()
            .collect();
        ValBag::assert_new(union.into_iter())
    }
    /// Take the intersection of two bags
    /// This impl is to avoid recompilation for every `ValArr<A, P>`, `ValArr<B, Q>` pair.
    pub fn intersect_impl(&self, rhs: &ValBag) -> ValBag {
        // Edge cases
        if rhs.is_empty() {
            return rhs.clone();
        } else if self.is_empty() {
            return self.clone();
        }
        let intersection: Vec<_> = self
            .iter_vals()
            .merge_join_by(rhs.iter_vals(), |l, r| l.as_ptr().cmp(&r.as_ptr()))
            .filter_map(|v| v.both().map(|(l, _)| l))
            .cloned()
            .collect();
        ValBag::assert_new(intersection.into_iter())
    }
}

impl<A: BagMarker, P> ValArr<A, P> {
    /// Forget any additional array type information, yielding a `ValBag`
    pub fn as_bag(&self) -> &ValBag<P> {
        self.coerce_ref()
    }
    /// Check whether an item is in this bag. If it is, return a reference.
    #[inline]
    pub fn contains<Q>(&self, item: *const NormalValue) -> Option<&ValId<Q>> {
        self.as_bag()
            .as_vals()
            .contains_impl(item)
            .map(ValId::coerce_ref)
    }
    /// Deduplicate this bag to yield a `ValSet`
    pub fn uniq(&self) -> ValSet<P> {
        let vals: Vec<_> = self.iter_vals().dedup().collect();
        ValSet::assert_new(vals.into_iter().cloned())
    }
    /// Merge two bags
    #[inline]
    pub fn merge<B: BagMarker>(&self, rhs: &ValArr<B, P>) -> ValBag<P> {
        self.as_bag()
            .as_vals()
            .merge_impl(rhs.coerce_ref())
            .coerce()
    }
    /// Take the intersection of two bags
    pub fn intersect<B: BagMarker>(&self, rhs: &ValArr<B, P>) -> ValArr<A, P> {
        self.as_bag()
            .as_vals()
            .intersect_impl(rhs.coerce_ref())
            .coerce()
    }
}

impl<A: SetMarker, P> ValArr<A, P> {
    /// Forget any additional array information, yielding a `ValSet`
    pub fn as_set(&self) -> &ValSet<P> {
        self.coerce_ref()
    }
    /// Take the union of two `ValSet`s
    pub fn union<B: SetMarker>(&self, rhs: &ValArr<B, P>) -> ValSet<P> {
        // Edge cases
        if rhs.is_empty() {
            return self.clone().coerce();
        } else if self.is_empty() {
            return rhs.clone().coerce();
        }
        let union: Vec<_> = self
            .iter_vals()
            .merge_join_by(rhs.iter_vals(), |l, r| l.as_ptr().cmp(&r.as_ptr()))
            .map(|v| v.reduce(|l, _| l))
            .cloned()
            .collect();
        ValSet::assert_new(union.into_iter())
    }
    /// Take the symmetric difference of two `ValSet`s
    pub fn diff<B: SetMarker>(&self, rhs: &ValArr<B, P>) -> ValSet<P> {
        if rhs.is_empty() {
            return self.clone().coerce();
        } else if self.is_empty() {
            return rhs.clone().coerce();
        }
        let diff: Vec<_> = self
            .iter_vals()
            .merge_join_by(rhs.iter_vals(), |l, r| l.as_ptr().cmp(&r.as_ptr()))
            .filter_map(|v| v.map_any(Some, Some).reduce(|_, _| None))
            .cloned()
            .collect();
        ValSet::assert_new(diff.into_iter())
    }
}

/// A bag, that is, a multiset (implemented as a sorted array) of ValIds
pub type ValBag<P = ()> = ValArr<Sorted, P>;

/// A bag, that is, a multiset (implemented as a sorted array) of types
pub type TyBag = VarBag<TypeValue>;

/// A set (implemented as a sorted, unique array) of ValIds
pub type ValSet<P = ()> = ValArr<Uniq, P>;

/// A set (implemented as a sorted, unique array) of types
pub type TySet = VarSet<TypeValue>;

impl ValArr {
    /// Create a `ValArr` from an exact size iterator over `ValId`s
    #[inline]
    pub fn new<I: Iterator<Item = ValId> + ExactSizeIterator>(vals: I) -> ValArr {
        Self::assert_new(vals)
    }
    /// Deduplicate an `Arc` to an array of `ValId`s to get a `ValArr`
    #[inline]
    pub fn dedup(ava: CachedArr<ValId>) -> ValArr {
        let dedup_ava = if ava.len() != 0 {
            ARRAY_CACHE.cache(ava)
        } else {
            ava // Don't waste cache space on an empty array
        };
        ValArr {
            arr: dedup_ava,
            variant: std::marker::PhantomData,
        }
    }
}

impl FromIterator<ValId> for ValArr {
    fn from_iter<I: IntoIterator<Item = ValId>>(iter: I) -> ValArr {
        let v: Vec<_> = iter.into_iter().collect();
        //TODO: optimize the case where size is known?
        Self::new(v.into_iter())
    }
}

impl FromIterator<ValId> for ValBag {
    fn from_iter<I: IntoIterator<Item = ValId>>(iter: I) -> ValBag {
        let v = iter.into_iter().sorted_by_key(ValId::as_ptr);
        Self::assert_new(v)
    }
}

impl FromIterator<ValId> for ValSet {
    fn from_iter<I: IntoIterator<Item = ValId>>(iter: I) -> ValSet {
        let mut v: Vec<_> = iter.into_iter().sorted_by_key(ValId::as_ptr).collect();
        v.dedup();
        Self::assert_new(v.into_iter())
    }
}

impl<V: Value> FromIterator<V> for VarArr<V> {
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> VarArr<V> {
        let v: Vec<ValId> = iter.into_iter().map(Value::into_val).collect();
        Self::assert_new(v.into_iter())
    }
}

impl<V: Value> FromIterator<V> for VarBag<V> {
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> VarBag<V> {
        let v = iter
            .into_iter()
            .map(Value::into_val)
            .sorted_by_key(ValId::as_ptr);
        Self::assert_new(v)
    }
}

impl<V: Value> FromIterator<V> for VarSet<V> {
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> VarSet<V> {
        let mut v: Vec<_> = iter
            .into_iter()
            .map(Value::into_val)
            .sorted_by_key(ValId::as_ptr)
            .collect();
        v.dedup();
        Self::assert_new(v.into_iter())
    }
}

impl<V: Value> FromIterator<VarId<V>> for VarArr<V> {
    fn from_iter<I: IntoIterator<Item = VarId<V>>>(iter: I) -> VarArr<V> {
        let v: Vec<ValId> = iter.into_iter().map(Into::into).collect();
        Self::assert_new(v.into_iter())
    }
}

impl<V: Value> FromIterator<VarId<V>> for VarBag<V> {
    fn from_iter<I: IntoIterator<Item = VarId<V>>>(iter: I) -> VarBag<V> {
        let v = iter
            .into_iter()
            .map(Into::into)
            .sorted_by_key(ValId::as_ptr);
        Self::assert_new(v)
    }
}

impl<V: Value> FromIterator<VarId<V>> for VarSet<V> {
    fn from_iter<I: IntoIterator<Item = VarId<V>>>(iter: I) -> VarSet<V> {
        let mut v: Vec<_> = iter
            .into_iter()
            .map(Into::into)
            .sorted_by_key(ValId::as_ptr)
            .collect();
        v.dedup();
        Self::assert_new(v.into_iter())
    }
}

impl<A, P> Index<usize> for ValArr<A, P> {
    type Output = ValId<P>;
    fn index(&self, ix: usize) -> &ValId<P> {
        if let Some(arr) = &self.arr {
            arr[ix].coerce_ref()
        } else {
            panic!("Indexed empty VarArr with index {}", ix)
        }
    }
}

impl<A, P> Deref for ValArr<A, P> {
    type Target = [ValId<P>];
    #[inline]
    fn deref(&self) -> &[ValId<P>] {
        self.coerce_slice()
    }
}

/// A reference-counted, hash-consed, typed array of values.
///
/// Implementation detail: Should not be constructable by the user!
#[derive(Clone, Eq, PartialEq)]
pub struct PrivateValArr(ThinArc<(), ValId>);

impl Deref for PrivateValArr {
    type Target = [ValId];
    fn deref(&self) -> &[ValId] {
        &self.0.slice
    }
}

impl Debug for PrivateValArr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(self.deref(), fmt)
    }
}

impl Hash for PrivateValArr {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        std::ptr::hash(self.deref(), hasher)
    }
}

/// A reference-counted array of values which is not necessarily hash-consed.
/// This wrapper is for the `Hash` implementation
#[derive(Clone, Eq, PartialEq)]
pub struct ArcValIdArr(pub Arc<HeaderSlice<HeaderWithLength<()>, [ValId]>>);

impl Caches<[ValId]> for ArcValIdArr {
    #[inline]
    fn can_collect(&self) -> bool {
        self.0.is_unique()
    }
}

impl Deref for ArcValIdArr {
    type Target = [ValId];
    fn deref(&self) -> &[ValId] {
        &self.0.slice
    }
}

impl Debug for ArcValIdArr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        Debug::fmt(self.deref(), fmt)
    }
}

impl Hash for ArcValIdArr {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.deref().hash(hasher)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::finite::{Finite, Index};
    use rand::{Rng, SeedableRng};
    use rand_xoshiro::Xoroshiro128PlusPlus as TestRng;

    /// Basic set operations and construction
    #[test]
    fn basic_set_test() {
        // Data generation
        let fv: Vec<ValId> = (0..10)
            .map(|f| ValId::from(Finite(f)))
            .cycle()
            .take(25)
            .collect();
        let fv2: Vec<ValId> = (5..16)
            .map(|f| ValId::from(Finite(f)))
            .cycle()
            .take(35)
            .collect();

        // Direct fully unsorted array construction
        let ua = ValArr::<()>::from_iter(fv.iter().cloned());
        ua.try_sorted().expect_err("This array is not sorted!");
        ua.try_set().expect_err("This array is not a set!");
        assert_eq!(ua.len(), 25);
        let ua2 = ValArr::<()>::from_iter(fv2.iter().cloned());
        ua2.try_sorted().expect_err("This array is not sorted!");
        ua2.try_set().expect_err("This array is not a set!");
        assert_eq!(ua2.len(), 35);

        // Direct fully unsorted bag construction
        let ub = ValBag::<()>::from_iter(fv.iter().cloned());
        assert_eq!(ub.len(), 25);
        assert_ne!(ub, ua);
        assert!(ub.is_sorted());
        assert!(!ub.is_set());
        let ub2 = ValBag::<()>::from_iter(fv2.iter().cloned());
        assert_eq!(ub2.len(), 35);
        assert_ne!(ub2, ua2);
        assert!(ub2.is_sorted());
        assert!(!ub2.is_set());

        // Direct fully unsorted set construction
        let us = ValSet::<()>::from_iter(fv.iter().cloned());
        assert_eq!(us.len(), 10);
        assert!(us.is_sorted());
        assert!(us.is_set());
        let us2 = ValSet::<()>::from_iter(fv2.iter().cloned());
        assert_eq!(us2.len(), 11);
        assert!(us2.is_sorted());
        assert!(us2.is_set());

        // Set operations
        assert_eq!(us.intersect(&us), us);
        assert_eq!(us.union(&us), us);
        assert_eq!(us2.intersect(&us2), us2);
        assert_eq!(us2.union(&us2), us2);
        let us3 = us.intersect(&us2);
        let us4 = us.union(&us2);
        assert_eq!(us2.intersect(&us), us3);
        assert_eq!(us2.union(&us), us4);
        assert_eq!(us4.intersect(&us), us);
        assert_eq!(us4.intersect(&us2), us2);

        // Bag operations
        assert_eq!(ub.intersect(&ub), ub);
        let ubdup = ub.merge(&ub);
        assert_eq!(ubdup.len(), 50);
        let ubu = ub.uniq();
        assert_eq!(ubu, us);
        let ubdup2 = ub2.merge(&ub2);
        assert_eq!(ubdup2.len(), 70);
        let ubu2 = ub2.uniq();
        assert_eq!(ubu2, us2);
        let ub3 = ub.merge(&ub2);
        assert_eq!(ub3.len(), 60);
        let ubdup3 = ubdup.merge(&ubdup2);
        assert_eq!(ubdup3.len(), 120);
        let ub3dup = ub3.merge(&ub3);
        assert_eq!(ubdup3, ub3dup);
        assert_eq!(ub3dup.uniq(), us4);
    }

    /// A stress-test of the `ValArr` family of structs on large arrays of random indices
    #[test]
    fn random_arrays_of_indices_construct_correctly() {
        const TEST_SEED: u64 = 0x56614ffa9e2a191d;
        const MAX_ARRAY_SIZE: usize = 100;
        const ARRAYS_TO_TEST: usize = 100;
        let mut rng = TestRng::seed_from_u64(TEST_SEED);

        // Data generation
        let finite_arrays: Vec<Vec<_>> = (0..ARRAYS_TO_TEST)
            .map(|_| {
                let length = rng.gen_range(0, MAX_ARRAY_SIZE);
                (0..length)
                    .map(|_| {
                        let fin = Finite(rng.gen());
                        fin.ix(rng.gen_range(0, fin.0)).unwrap()
                    })
                    .collect()
            })
            .collect();

        // Basic construction test
        let finite_valarrs_uncached: Vec<_> = finite_arrays
            .iter()
            .map(|arr| arr.iter().map(|ix| ix.clone().into()).collect_vec())
            .collect();
        let finite_valarrs: Vec<_> = finite_valarrs_uncached
            .iter()
            .map(|arr| ValArr::new(arr.iter().cloned()))
            .collect();
        let finite_valarrs_2: Vec<_> = finite_arrays
            .iter()
            .map(|arr| ValArr::new(arr.iter().map(|ix| ix.clone().into())))
            .collect();
        let finite_valarrs_3: Vec<VarArr<Index>> = finite_arrays
            .iter()
            .map(|arr| arr.iter().cloned().collect())
            .collect();
        assert_eq!(finite_valarrs, finite_valarrs_2);
        assert_eq!(finite_valarrs.deref(), finite_valarrs_2.deref());

        // Basic identity tests
        for i in 0..finite_arrays.len() {
            assert_eq!(
                finite_valarrs[i].deref() as *const [ValId],
                finite_valarrs_2[i].deref() as *const [ValId],
                "Failure at index {}",
                i
            );
            assert_eq!(
                finite_valarrs[i].as_ptr(),
                finite_valarrs[i].deref() as *const [ValId],
                "Failure at index {}",
                i
            );
            assert_eq!(
                finite_valarrs_2[i].deref() as *const [ValId],
                finite_valarrs_2[i].as_ptr(),
                "Failure at index {}",
                i
            );
            assert_eq!(
                finite_valarrs[i].as_ptr(),
                finite_valarrs_3[i].as_ptr(),
                "Failure at index {}",
                i
            );
            assert_eq!(
                finite_valarrs[i].as_ptr(),
                finite_valarrs_3[i].as_valarr().as_ptr(),
                "Failure at index {}",
                i
            );
            assert_eq!(
                finite_valarrs[i].as_ptr(),
                finite_valarrs_3[i].as_valarr().deref(),
                "Failure at index {}",
                i
            );
            assert_ne!(
                finite_valarrs[i].deref() as *const [ValId],
                finite_valarrs_uncached[i].deref() as *const [ValId],
                "Failure at index {}",
                i
            );
            assert_eq!(
                finite_valarrs[i].deref(),
                finite_valarrs_uncached[i].deref(),
                "Failure at index {}",
                i
            );
        }

        // Sorting tests:
        let sorted_valarrs: Vec<_> = finite_valarrs.iter().map(|v| v.sorted()).collect();
        for (i, v) in sorted_valarrs.iter().enumerate() {
            if v.len() > 2 {
                for j in 0..(v.len() - 1) {
                    assert!(
                        v[j].as_ptr() <= v[j + 1].as_ptr(),
                        "Array {} is not sorted: out-of-order elements {}@{:?}, {}@{:?} at index {}",
                        i,
                        v[j],
                        v[j].as_ptr(),
                        v[j + 1],
                        v[j + 1].as_ptr(),
                        j
                    )
                }
            }
            assert!(
                v.is_sorted(),
                "Array {} is sorted, but does not report itself sorted!\nINDEX:\n{:#?}",
                i,
                v
            );
        }
    }
}
