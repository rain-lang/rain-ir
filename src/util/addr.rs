/*!
Utilities for low-level manipulation of memory
*/
use hashbrown::{hash_map::RawEntryMut, HashMap};
use std::hash::{BuildHasher, Hash, Hasher};

/// A trait for data structures which have a known lookup address, *and are hashed by this address only*
pub trait HasAddr {
    /// Get the lookup address of this value
    fn raw_addr(&self) -> usize;
    /// Get the hash of this value's address
    fn addr_hash<S: BuildHasher>(&self, hash_builder: &S) -> u64 {
        let mut hasher = hash_builder.build_hasher();
        self.hash_addr(&mut hasher);
        hasher.finish()
    }
    /// Hash this value's address
    fn hash_addr<H: Hasher>(&self, hasher: &mut H) {
        self.raw_addr().hash(hasher);
    }
}

impl<T> HasAddr for *const T {
    #[inline(always)]
    fn raw_addr(&self) -> usize {
        *self as usize
    }
}

impl<T> HasAddr for *mut T {
    #[inline(always)]
    fn raw_addr(&self) -> usize {
        *self as usize
    }
}

impl<T: HasAddr> HasAddr for Option<T> {
    #[inline(always)]
    fn raw_addr(&self) -> usize {
        self.as_ref().map(HasAddr::raw_addr).unwrap_or_default()
    }
}

impl<T: HasAddr> HasAddr for &T {
    #[inline(always)]
    fn raw_addr(&self) -> usize {
        (**self).raw_addr()
    }
}

impl<T: HasAddr> HasAddr for &mut T {
    #[inline(always)]
    fn raw_addr(&self) -> usize {
        (**self).raw_addr()
    }
}

/// A trait for data structures which can be looked up by address
pub trait AddrLookup<K, V> {
    /// Lookup a value by address
    #[inline]
    fn lookup<A: HasAddr>(&self, value: &A) -> Option<(&K, &V)> {
        self.lookup_addr(value.raw_addr())
    }
    /// Lookup an address
    fn lookup_addr(&self, addr: usize) -> Option<(&K, &V)>;
}

impl<A: HasAddr, T, S: BuildHasher> AddrLookup<A, T> for HashMap<A, T, S> {
    #[inline]
    fn lookup_addr(&self, addr: usize) -> Option<(&A, &T)> {
        let mut hasher = self.hasher().build_hasher();
        addr.hash(&mut hasher);
        let hash = hasher.finish();
        self.raw_entry()
            .from_hash(hash, |value| value.raw_addr() == addr)
    }
}

/// A trait for data structures which can be mutably looked up by address
pub trait AddrLookupMut<K, V> {
    /// Lookup a value by address
    #[inline]
    fn lookup_mut<A: HasAddr, F>(&mut self, value: &A, on_empty: F) -> Option<(&K, &mut V)>
    where
        F: FnOnce() -> Option<(K, V)>,
    {
        self.lookup_addr_mut(value.raw_addr(), on_empty)
    }
    /// Lookup a value by address, or insert it
    #[inline]
    fn lookup_or_insert<A: HasAddr, F>(&mut self, value: &A, on_empty: F) -> (&K, &mut V)
    where
        F: FnOnce() -> (K, V),
    {
        self.lookup_addr_or_insert(value.raw_addr(), on_empty)
    }
    /// Lookup an address or insert
    fn lookup_addr_or_insert<F>(&mut self, addr: usize, on_empty: F) -> (&K, &mut V)
    where
        F: FnOnce() -> (K, V),
    {
        self.lookup_addr_mut(addr, || Some(on_empty()))
            .expect("Insertion always succeeds")
    }
    /// Lookup an address
    fn lookup_addr_mut<F>(&mut self, addr: usize, on_empty: F) -> Option<(&K, &mut V)>
    where
        F: FnOnce() -> Option<(K, V)>;
}

impl<A: HasAddr + Hash + Eq, T, S: BuildHasher> AddrLookupMut<A, T> for HashMap<A, T, S> {
    #[inline]
    fn lookup_addr_mut<F>(&mut self, addr: usize, on_empty: F) -> Option<(&A, &mut T)>
    where
        F: FnOnce() -> Option<(A, T)>,
    {
        let mut hasher = self.hasher().build_hasher();
        addr.hash(&mut hasher);
        let hash = hasher.finish();
        let (key, value) = match self
            .raw_entry_mut()
            .from_hash(hash, |value| value.raw_addr() == addr)
        {
            RawEntryMut::Occupied(o) => o.into_key_value(),
            RawEntryMut::Vacant(v) => {
                let (key, value) = on_empty()?;
                v.insert_hashed_nocheck(hash, key, value)
            }
        };
        Some((key, value))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::primitive::logical::{BOOL_TY, FALSE, TRUE};
    use crate::value::Value;

    #[test]
    fn basic_hasaddr_properties() {
        let null: *const () = std::ptr::null();
        assert_eq!(null.raw_addr(), 0x0);
        assert_ne!(BOOL_TY.raw_addr(), 0x0);
        let mut map = HashMap::new();
        map.insert(true.into_val(), 73);
        map.insert(false.into_val(), 53);
        assert_eq!(map.lookup(&*TRUE), Some((TRUE.as_val(), &73)));
        assert_eq!(map.lookup(&*FALSE), Some((FALSE.as_val(), &53)));
        assert_eq!(map.lookup(&*BOOL_TY), None);
    }
}
