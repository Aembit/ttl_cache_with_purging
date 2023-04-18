use std::{borrow::Borrow, collections::HashMap, hash::Hash, time::Instant};

pub struct Cache<K, V> {
    map: HashMap<K, CacheEntry<V>>,
}

impl<K, V> Cache<K, V> {
    pub fn new() -> Self {
        Cache {
            map: HashMap::new(),
        }
    }
}

impl<K, V> Cacheable<K, V> for Cache<K, V>
where
    K: Eq + Hash,
{
    fn insert(&mut self, key: K, val: V, expires_at: Instant) {
        self.map.insert(key, CacheEntry { val, expires_at });
    }

    fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.map.get(key) {
            Some(entry) if entry.expires_at > Instant::now() => Some(&entry.val),
            _ => None,
        }
    }

    fn purge_expired(&mut self) {
        let now = Instant::now();
        self.map.retain(|_k, v| now < v.expires_at)
    }
}

/// These actions are implemented as a trait to improve testability.
pub trait Cacheable<K, V> {
    fn insert(&mut self, key: K, val: V, expires_at: Instant);

    fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized + 'static;

    fn purge_expired(&mut self);
}

struct CacheEntry<V> {
    val: V,
    expires_at: Instant,
}

#[cfg(test)]
pub(crate) mod test_helpers {
    use std::{borrow::Borrow, hash::Hash, time::Instant};

    use mockall::mock;

    use super::Cacheable;

    mock! {
        pub Cache<K: 'static, V: 'static> {}

        impl<K , V> Cacheable<K, V> for Cache<K, V> {
            fn purge_expired(&mut self) {}

            fn get<Q>(&self, key: &Q) -> Option<&'static V>
            where
                K: Borrow<Q>,
                Q: Hash + Eq + ?Sized + 'static, {}

            fn insert(&mut self, key: K, val: V, expires_at: Instant) {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use lazy_static::lazy_static;

    use crate::cache::{Cache, Cacheable};

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
        let mut cache = Cache::new();

        // Act
        cache.insert("key", "val", *UNEXPIRED_INSTANT);

        // Assert
        assert_eq!(*cache.get("key").unwrap(), "val");
    }

    #[test]
    fn given_an_entry_in_the_cache_when_adding_an_entry_with_the_same_key_then_the_value_is_overwritten(
    ) {
        // Arrange
        let mut cache = Cache::new();
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
        let mut cache = Cache::new();
        let key = "key";
        cache.insert(key, "val", *EXPIRED_INSTANT);

        // Act
        let val = cache.get(key);

        // Assert
        assert!(val.is_none())
    }

    #[test]
    fn given_a_mixture_of_expired_entries_and_active_entries_when_deleting_the_expired_entries_then_the_expired_entries_are_removed_and_the_active_entries_remain(
    ) {
        // Arrange
        let mut cache = Cache::new();
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
