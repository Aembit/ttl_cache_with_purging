//! Standard cache operations.
use std::{borrow::Borrow, collections::HashMap, hash::Hash, time::Instant};

/// An instance of a cache.
#[derive(Default)]
pub struct TtlCache<K, V> {
    map: HashMap<K, CacheEntry<V>>,
}

struct CacheEntry<V> {
    val: V,
    expires_at: Instant,
}

impl<K, V> TtlCache<K, V>
where
    K: Eq + Hash,
{
    /// Creates a new cache instance.
    pub fn new() -> Self {
        TtlCache {
            map: HashMap::new(),
        }
    }

    /// Adds a new value to the cache that will expire at the specified instant.
    pub fn insert(&mut self, key: K, val: V, expires_at: Instant) {
        self.map.insert(key, CacheEntry { val, expires_at });
    }

    /// Retrieves an unexpired value from the cache.
    ///
    /// Expired entries will return `None`.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get_value_and_expiration(key)
            .map(|(val, _expires_at)| val)
    }

    /// Retrieves an unexpired value from the cache, along with the expiration.
    ///
    /// Expired entries will return `None`.
    pub fn get_value_and_expiration<Q>(&self, key: &Q) -> Option<(&V, Instant)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map
            .get(key)
            .filter(|e| e.expires_at > Instant::now())
            .map(|e| (&e.val, e.expires_at))
    }
}

/// Operations relating to purging expired entries.
///
/// This is extracted to make purge testing simpler.
pub trait Purgeable {
    /// Purges expired entries from the cache.
    fn purge_expired(&mut self);
}

impl<K, V> Purgeable for TtlCache<K, V> {
    fn purge_expired(&mut self) {
        let now = Instant::now();
        self.map.retain(|_k, v| now < v.expires_at)
    }
}

#[cfg(test)]
pub(crate) mod test_helpers {

    use super::Purgeable;

    #[derive(Default)]
    pub(crate) struct SpyCache {
        pub purge_expired_called: bool,
    }

    impl Purgeable for SpyCache {
        fn purge_expired(&mut self) {
            self.purge_expired_called = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use lazy_static::lazy_static;

    use crate::cache::{Purgeable, TtlCache};

    lazy_static! {
        static ref UNEXPIRED_INSTANT: Instant = Instant::now()
            .checked_add(Duration::from_secs(86400))
            .unwrap();
        static ref EXPIRED_INSTANT: Instant =
            Instant::now().checked_sub(Duration::from_secs(10)).unwrap();
    }

    #[test]
    fn when_adding_an_active_entry_to_the_cache_then_it_is_present() {
        // Arrange
        let mut cache = TtlCache::new();
        let key = "key";
        let val = "val";

        // Act
        cache.insert(key, val, *UNEXPIRED_INSTANT);

        // Assert
        assert_eq!(*cache.get(key).unwrap(), val);
        assert_eq!(*cache.get_value_and_expiration(key).unwrap().0, val);
    }

    #[test]
    fn given_an_entry_in_the_cache_when_adding_an_entry_with_the_same_key_then_the_value_is_overwritten(
    ) {
        // Arrange
        let mut cache = TtlCache::new();
        let key = "key";
        cache.insert(
            key,
            "val1",
            Instant::now()
                .checked_add(Duration::from_secs(1000))
                .unwrap(),
        );
        let value_to_overwrite_with = "val2";

        // Act
        cache.insert(key, value_to_overwrite_with, *UNEXPIRED_INSTANT);

        // Assert
        assert_eq!(*cache.get(key).unwrap(), value_to_overwrite_with);
    }

    #[test]
    fn given_an_expired_entry_in_the_cache_when_retrieving_it_then_it_is_not_returned() {
        // Arrange
        let mut cache = TtlCache::new();
        let key = "key";
        cache.insert(key, "val", *EXPIRED_INSTANT);

        // Act
        let val = cache.get(key);
        let val_with_expiration = cache.get_value_and_expiration(key);

        // Assert
        assert!(val.is_none());
        assert!(val_with_expiration.is_none());
    }

    #[test]
    fn given_a_mixture_of_expired_entries_and_active_entries_when_deleting_the_expired_entries_then_the_expired_entries_are_removed_and_the_active_entries_remain(
    ) {
        // Arrange
        let mut cache = TtlCache::new();
        let unexpired = "unexpired";
        let expired = "expired";
        cache.insert(unexpired, "val1", *UNEXPIRED_INSTANT);
        cache.insert(expired, "val2", *EXPIRED_INSTANT);

        // Act
        cache.purge_expired();

        // Assert
        assert!(cache.map.get(unexpired).is_some());
        assert!(cache.map.get(expired).is_none());
    }
}
