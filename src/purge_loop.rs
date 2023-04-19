use std::{hash::Hash, sync::Arc};

use tokio::{sync::RwLock, time::Interval};

use crate::cache::Cacheable;

/// Kick-off a loop that will purge expired entries from the cache at a
/// specified interval.
pub fn start_purge_loop<C, K, V>(cache: Arc<RwLock<C>>, mut purge_interval: Interval)
where
    C: Cacheable<K, V> + Send + Sync + 'static,
    K: Eq + Hash + Send + Sync,
    V: Send + Sync,
{
    tokio::task::spawn(async move {
        loop {
            // Note that the first tick is instantaneous.
            purge_interval.tick().await;
            cache.write().await.purge_expired();
        }
    });
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use tokio::sync::RwLock;
    use tokio::time::{interval, sleep};

    use crate::cache::test_helpers::SpyCache;
    use crate::purge_loop::start_purge_loop;

    #[tokio::test]
    async fn when_the_purge_loop_runs_then_the_cache_deletes_expired_entries() {
        // Arrange
        let cache = Arc::new(RwLock::new(SpyCache::default()));

        // Act
        start_purge_loop::<SpyCache, String, String>(
            cache.clone(),
            interval(Duration::from_secs(10000)),
        );

        // Assert
        sleep(Duration::from_millis(1)).await;
        assert!(cache.write().await.purge_expired_called);
    }
}
