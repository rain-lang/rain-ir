/*!
Reference-counted, hash-consed, typed arrays of values
*/

use super::{ValId, Value, VarId};
use crate::typing::TypeValue;
use crate::util::hash_cache::{Cache, Caches};
use itertools::Itertools;
use lazy_static::lazy_static;
use ref_cast::RefCast;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::{Deref, Index};
use triomphe::{Arc, HeaderSlice, HeaderWithLength, ThinArc};

lazy_static! {
    /// A cache for arrays of values
    pub static ref ARRAY_CACHE: Cache<ValId, ArcValIdArr> = Cache::default();
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
#[derive(Debug, Eq, Hash, RefCast)]
#[repr(transparent)]
pub struct VarArr<V> {
    //TODO: remove Option when I figure out how to support empty slices...
    arr: Option<PrivateValArr>,
    variant: std::marker::PhantomData<V>,
}

impl<V> Clone for VarArr<V> {
    fn clone(&self) -> VarArr<V> {
        VarArr {
            arr: self.arr.clone(),
            variant: self.variant,
        }
    }
}

impl<V> Default for VarArr<V> {
    /// Get an empty `VarArr`
    fn default() -> VarArr<V> {
        VarArr::EMPTY_SELF
    }
}

impl<U, V> PartialEq<VarArr<U>> for VarArr<V> {
    fn eq(&self, other: &VarArr<U>) -> bool {
        self.arr == other.arr
    }
}

/// The unique empty array of `ValId`s. Temporary hack...
static UNIQUE_EMPTY_ARRAY: [ValId; 0] = [];

impl<V> VarArr<V> {
    /// This type as an empty array, for use in `const` contexts
    pub const EMPTY_SELF: VarArr<V> = VarArr {
        arr: None,
        variant: std::marker::PhantomData,
    };
    /// Get the length of this array
    #[inline]
    pub fn len(&self) -> usize {
        if let Some(arr) = &self.arr {
            arr.len()
        } else {
            0
        }
    }
    /// Check whether this array is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.arr.is_none()
    }
    /// Get this array as an array of ValIds
    #[inline]
    pub fn as_vals(&self) -> &VarArr<ValIdMarker> {
        self.coerce()
    }
    /// Get this array as a slice of ValIds
    #[inline]
    pub fn as_slice(&self) -> &[ValId] {
        if let Some(arr) = &self.arr {
            &arr
        } else {
            &UNIQUE_EMPTY_ARRAY
        }
    }
    /// Get this array as a pointer to an array of ValIds
    #[inline]
    pub fn as_ptr(&self) -> *const [ValId] {
        if let Some(arr) = &self.arr {
            arr.deref()
        } else {
            &UNIQUE_EMPTY_ARRAY
        }
    }
    /// Check if this array is address-sorted
    #[inline]
    pub fn is_sorted(&self) -> bool {
        if let Some(arr) = &self.arr {
            is_sorted::IsSorted::is_sorted_by_key(&mut arr.iter(), |v| v.as_ptr())
        } else {
            true
        }
    }
    /// Check that this array is address-sorted in a strictly increasing fashion
    #[inline]
    pub fn is_set(&self) -> bool {
        if let Some(arr) = &self.arr {
            arr.windows(2).all(|w| w[0].as_ptr() < w[1].as_ptr())
        } else {
            true
        }
    }
    /// If this array is address-sorted, return it as a sorted array. If not, fail.
    #[inline]
    pub fn try_sorted(&self) -> Result<&VarBag<V>, ()> {
        if self.is_sorted() {
            Ok(RefCast::ref_cast(&self.arr))
        } else {
            Err(())
        }
    }
    /// If this array is strictly address-sorted, return it as a set. If not, fail.
    #[inline]
    pub fn try_set(&self) -> Result<&VarSet<V>, ()> {
        if self.is_set() {
            Ok(RefCast::ref_cast(&self.arr))
        } else {
            Err(())
        }
    }
    /// Clone this array as an array of ValIds
    #[inline]
    pub fn clone_vals(&self) -> VarArr<ValIdMarker> {
        VarArr {
            arr: self.arr.clone(),
            variant: std::marker::PhantomData,
        }
    }
    /// Create a `VarArr` from an exact size iterator over `ValId`s, asserting it is of the desired type
    #[inline]
    fn assert_new<I: Iterator<Item = ValId> + ExactSizeIterator>(vals: I) -> VarArr<V> {
        if vals.len() == 0 {
            // Avoid empty array bugs
            return VarArr {
                arr: None,
                variant: std::marker::PhantomData,
            };
        }
        Self::dedup_and_assert(ArcValIdArr(Arc::from_header_and_iter(
            HeaderWithLength::new((), vals.len()),
            vals,
        )))
    }
    /// Deduplicate an `Arc` to an array of `ValId`s, and assert this array is of the desired type
    #[inline]
    fn dedup_and_assert(ava: ArcValIdArr) -> VarArr<V> {
        let dedup_ava = ARRAY_CACHE.cache(ava);
        VarArr {
            arr: Some(PrivateValArr(Arc::into_thin(dedup_ava.0))),
            variant: std::marker::PhantomData,
        }
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
    pub fn sorted(&self) -> VarBag<V> {
        // Special case (TODO: consider performance implications...)
        if self.is_sorted() {
            return VarBag {
                arr: self.arr.clone(),
                variant: std::marker::PhantomData,
            };
        }
        VarBag::assert_new(self.iter_vals().sorted_by_key(|v| v.as_ptr()).cloned())
    }
    /// Address-sort this container *with* deduplication, yielding a `VarSet`
    #[inline]
    pub fn set(&self) -> VarSet<V> {
        let mut source: Vec<_> = self.iter_vals().sorted_by_key(|v| v.as_ptr()).collect();
        source.dedup();
        VarSet::assert_new(source.into_iter().cloned())
    }
    /// Coerce this container into a set of `VarId<V>`, asserting the predicate holds *for each container element*!
    #[inline]
    fn coerce<U>(&self) -> &VarArr<U> {
        RefCast::ref_cast(&self.arr)
    }
    /// Coerce this container into a slice of `VarId<V>`, asserting that the predicate holds *for each container element*!
    /// While this method can be called incorrectly, it is *safe* as regardless of `V`, all `VarId<V>` are guaranteed to have
    /// the same representation.
    #[inline]
    fn coerce_slice<U>(&self) -> &[VarId<U>] {
        unsafe {
            std::mem::transmute(if let Some(arr) = &self.arr {
                arr.deref()
            } else {
                &UNIQUE_EMPTY_ARRAY[..]
            })
        }
    }
}

/// A marker for an array of ValIds
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ValIdMarker;

/// A marker for a sorted array of a given value type
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Sorted<T>(pub std::marker::PhantomData<T>);

/// A trait implemented by markers which expose the `Bag` API
pub trait BagMarker {}

impl<T> BagMarker for Sorted<T> {}

/// A marker for a sorted array of unique elements of a given value type
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Uniq<T>(pub std::marker::PhantomData<T>);

/// A trait implemented by markers which expose the `Set` API
pub trait SetMarker: BagMarker {}

impl<T> SetMarker for Uniq<T> {}

impl<T> BagMarker for Uniq<T> {}

impl<T> Sorted<T> {
    /// Create a new sorted array marker
    #[inline]
    pub fn new() -> Sorted<T> {
        Sorted(std::marker::PhantomData)
    }
}

impl<T> Uniq<T> {
    /// Create a new sorted array marker
    #[inline]
    pub fn new() -> Sorted<T> {
        Sorted(std::marker::PhantomData)
    }
}

/// An array of ValIds
pub type ValArr = VarArr<ValIdMarker>;

/// An array of types
pub type TyArr = VarArr<TypeValue>;

/// A bag (implemented as a sorted array) of `rain` values
pub type VarBag<V> = VarArr<Sorted<V>>;

/// A set (implemented as a sorted, unique array) of `rain` values
pub type VarSet<V> = VarArr<Uniq<V>>;

impl<V> VarBag<V> {
    /// Forget the information that this is in fact a `VarBag`, yielding a `VarArr`
    pub fn as_arr(&self) -> &VarArr<V> {
        RefCast::ref_cast(&self.arr)
    }
    /// Deduplicate this `VarBag` to yield a `VarSet`
    pub fn uniq(&self) -> VarSet<V> {
        let vals: Vec<_> = self.iter_vals().dedup().collect();
        VarSet::assert_new(vals.into_iter().cloned())
    }
}

impl<V: BagMarker> VarArr<V> {
    /// Merge two `VarBag`s
    pub fn merge(&self, rhs: &VarArr<V>) -> VarArr<V> {
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
        Self::assert_new(union.into_iter())
    }
    /// Take the intersection of two `VarBag`s
    pub fn intersect(&self, rhs: &VarArr<V>) -> VarArr<V> {
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
        Self::assert_new(intersection.into_iter())
    }
}

impl<V> VarSet<V> {
    /// Forget the information that this is in fact a `VarSet`, yielding a `VarArr`
    pub fn as_arr(&self) -> &VarArr<V> {
        self.coerce()
    }
    /// Forget the information that this is in fact a `VarSet`, yielding a `VarBag`
    pub fn as_multiset(&self) -> &VarBag<V> {
        self.coerce()
    }
}

impl<V: SetMarker> VarArr<V> {
    /// Take the union of two `VarSet`s
    pub fn union(&self, rhs: &VarArr<V>) -> VarArr<V> {
        // Edge cases
        if rhs.is_empty() {
            return self.clone();
        } else if self.is_empty() {
            return rhs.clone();
        }
        let union: Vec<_> = self
            .iter_vals()
            .merge_join_by(rhs.iter_vals(), |l, r| l.as_ptr().cmp(&r.as_ptr()))
            .map(|v| v.reduce(|l, _| l))
            .cloned()
            .collect();
        Self::assert_new(union.into_iter())
    }
}

/// A bag, that is, a multiset (implemented as a sorted array) of ValIds
pub type ValBag = VarBag<ValIdMarker>;

/// A bag, that is, a multiset (implemented as a sorted array) of types
pub type TyBag = VarBag<TypeValue>;

/// A set (implemented as a sorted, unique array) of ValIds
pub type ValSet = VarSet<ValIdMarker>;

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
    pub fn dedup(ava: ArcValIdArr) -> ValArr {
        Self::dedup_and_assert(ava)
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
        let v: Vec<ValId> = iter.into_iter().map(Into::into).collect();
        Self::assert_new(v.into_iter())
    }
}

impl<V: Value> FromIterator<V> for VarBag<V> {
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> VarBag<V> {
        let v = iter
            .into_iter()
            .map(|v| v.into())
            .sorted_by_key(ValId::as_ptr);
        Self::assert_new(v)
    }
}

impl<V: Value> FromIterator<V> for VarSet<V> {
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> VarSet<V> {
        let mut v: Vec<_> = iter
            .into_iter()
            .map(|v| v.into())
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

impl<V: Value> Index<usize> for VarArr<V> {
    type Output = VarId<V>;
    fn index(&self, ix: usize) -> &VarId<V> {
        if let Some(arr) = &self.arr {
            RefCast::ref_cast(&arr[ix].0)
        } else {
            panic!("Indexed empty VarArr with index {}", ix)
        }
    }
}

impl Index<usize> for VarArr<ValIdMarker> {
    type Output = ValId;
    fn index(&self, ix: usize) -> &ValId {
        if let Some(arr) = &self.arr {
            &arr[ix]
        } else {
            panic!("Indexed empty ValArr with index {}", ix)
        }
    }
}

impl<V: Value> Index<usize> for VarArr<Sorted<V>> {
    type Output = VarId<V>;
    fn index(&self, ix: usize) -> &VarId<V> {
        if let Some(arr) = &self.arr {
            RefCast::ref_cast(&arr[ix].0)
        } else {
            panic!("Indexed empty VarArr with index {}", ix)
        }
    }
}

impl Index<usize> for VarArr<Sorted<ValIdMarker>> {
    type Output = ValId;
    fn index(&self, ix: usize) -> &ValId {
        if let Some(arr) = &self.arr {
            &arr[ix]
        } else {
            panic!("Indexed empty ValArr with index {}", ix)
        }
    }
}

impl Deref for ValArr {
    type Target = [ValId];
    fn deref(&self) -> &[ValId] {
        if let Some(arr) = &self.arr {
            &arr
        } else {
            &UNIQUE_EMPTY_ARRAY
        }
    }
}

impl Deref for ValBag {
    type Target = [ValId];
    fn deref(&self) -> &[ValId] {
        if let Some(arr) = &self.arr {
            &arr
        } else {
            &UNIQUE_EMPTY_ARRAY
        }
    }
}

impl Deref for ValSet {
    type Target = [ValId];
    fn deref(&self) -> &[ValId] {
        if let Some(arr) = &self.arr {
            &arr
        } else {
            &UNIQUE_EMPTY_ARRAY
        }
    }
}

impl<V: Value> Deref for VarArr<V> {
    type Target = [VarId<V>];
    fn deref(&self) -> &[VarId<V>] {
        self.coerce_slice()
    }
}

impl<V: Value> Deref for VarBag<V> {
    type Target = [VarId<V>];
    fn deref(&self) -> &[VarId<V>] {
        self.coerce_slice()
    }
}

impl<V: Value> Deref for VarSet<V> {
    type Target = [VarId<V>];
    fn deref(&self) -> &[VarId<V>] {
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

impl Caches<ValId> for ArcValIdArr {
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
        let ua = ValArr::from_iter(fv.iter().cloned());
        ua.try_sorted().expect_err("This array is not sorted!");
        ua.try_set().expect_err("This array is not a set!");
        assert_eq!(ua.len(), 25);
        let ua2 = ValArr::from_iter(fv2.iter().cloned());
        ua2.try_sorted().expect_err("This array is not sorted!");
        ua2.try_set().expect_err("This array is not a set!");
        assert_eq!(ua2.len(), 35);

        // Direct fully unsorted bag construction
        let ub = ValBag::from_iter(fv.iter().cloned());
        assert_eq!(ub.len(), 25);
        assert_ne!(ub, ua);
        assert!(ub.is_sorted());
        assert!(!ub.is_set());
        let ub2 = ValBag::from_iter(fv2.iter().cloned());
        assert_eq!(ub2.len(), 35);
        assert_ne!(ub2, ua2);
        assert!(ub2.is_sorted());
        assert!(!ub2.is_set());

        // Direct fully unsorted set construction
        let us = ValSet::from_iter(fv.iter().cloned());
        assert_eq!(us.len(), 10);
        assert!(us.is_sorted());
        assert!(us.is_set());
        let us2 = ValSet::from_iter(fv2.iter().cloned());
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
                finite_valarrs_3[i].as_vals().as_ptr(),
                "Failure at index {}",
                i
            );
            assert_eq!(
                finite_valarrs[i].as_ptr(),
                finite_valarrs_3[i].as_vals().deref(),
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
