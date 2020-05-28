/*!
A cache for hash-consing values
*/

use ahash::RandomState;
use dashmap::DashMap;
use std::borrow::Borrow;
use std::hash::{BuildHasher, Hash};
use triomphe::Arc;

/// A cache for values of type `T`
#[derive(Debug)]
pub struct Cache<T: Hash + Eq, S: BuildHasher + Clone = RandomState> {
    /// The set of cached values
    cache: DashMap<Arc<T>, (), S>,
}

impl<T: Eq + Hash> Cache<T> {
    /// Create a new, empty cache
    pub fn new() -> Cache<T> {
        Cache {
            cache: DashMap::new(),
        }
    }
}

impl<T: Eq + Hash, S: BuildHasher + Clone + Default> Default for Cache<T, S> {
    fn default() -> Cache<T, S> {
        Cache {
            cache: DashMap::default(),
        }
    }
}

impl<T: Eq + Hash, S: BuildHasher + Clone> Cache<T, S> {
    /**
    Attempt to cache a value. If already cached, return the corresponding `Arc`

    # Example
    ```rust
    use rain_lang::util::hash_cache::Cache;
    use triomphe::Arc;
    let int_cache = Cache::<u64>::new();

    let cached_32 = int_cache.cache(32);
    let arc_32 = Arc::new(32);
    // These are different allocations!
    assert!(!Arc::ptr_eq(&arc_32, &cached_32));

    // We can use the cache to de-duplicate allocations
    let dedup_32 = int_cache.cache(arc_32.clone());
    assert!(Arc::ptr_eq(&dedup_32, &cached_32));

    // Similarly, we'll get the same `Arc` if we insert the value again
    let new_32 = int_cache.cache(32);
    assert!(Arc::ptr_eq(&new_32, &cached_32));
    
    // We can also insert an `Arc` from the get-go:
    let arc_44 = Arc::new(44);
    let cached_44 = int_cache.cache(arc_44.clone());
    assert!(Arc::ptr_eq(&arc_44, &cached_44));
    // New insertions are also deduplicated:
    let dedup_44 = int_cache.cache(44);
    assert!(Arc::ptr_eq(&arc_44, &dedup_44));
    ```
    */
    pub fn cache<Q>(&self, value: Q) -> Arc<T>
    where
        Arc<T>: Borrow<Q>,
        Q: Into<Arc<T>> + Hash + Eq,
    {
        // Read-lock first!
        // TODO: profile and see if this actually even helps efficiency at all
        if let Some(cached) = self.cache.get(&value) {
            return cached.key().clone();
        }
        self.cache.entry(value.into()).or_default().key().clone()
    }
    /**
    Garbage-collect a given cache. Return how many values were collected.

    # Example
    ```rust
    use rain_lang::util::hash_cache::Cache;
    use triomphe::Arc;
    let int_cache = Cache::<u64>::new();
    
    // Let's stick 2 used values and 3 unused values into the cache:
    let used_1 = int_cache.cache(77);
    let used_2 = int_cache.cache(88);
    int_cache.cache(99);
    int_cache.cache(500);
    int_cache.cache(81);
    // We can see that at this point there are 5 things in the cache:
    assert_eq!(int_cache.len(), 5);
    // Now, let's garbage collect the cache, which should bring us down 3 things:
    assert_eq!(int_cache.gc(), 3);
    // And we have 2 things left:
    assert_eq!(int_cache.len(), 2);
    assert!(Arc::ptr_eq(&used_1, &int_cache.cache(77)));
    assert!(Arc::ptr_eq(&used_2, &int_cache.cache(88)));
    ```
    */
    pub fn gc(&self) -> usize {
        let mut collected = 0;
        self.cache.retain(|arc, _| {
            if arc.is_unique() {
                collected += 1;
                false
            } else {
                true
            }
        });
        collected
    }

    /**
    Compute how many items are in a given cache.

    # Example
    ```rust
    use rain_lang::util::hash_cache::Cache;
    let int_cache = Cache::<u64>::new();
    assert_eq!(int_cache.len(), 0);
    int_cache.cache(10);
    assert_eq!(int_cache.len(), 1);
    int_cache.cache(20);
    assert_eq!(int_cache.len(), 2);
    // Since 10 is already in the cache, this is a no-op:
    int_cache.cache(10);
    assert_eq!(int_cache.len(), 2);
    ```
     */
    pub fn len(&self) -> usize {
        self.cache.len()
    }
}
