//! Strategies for purging expired cache entries.
use std::sync::Arc;

use tokio::{sync::RwLock, time::Interval};

use crate::cache::Purgeable;

/// Kick-off a background thread that will purge expired entries from the cache at the
/// specified interval.
pub fn start_periodic_purge<P>(cache: Arc<RwLock<P>>, mut purge_interval: Interval)
where
    P: Purgeable + Send + Sync + 'static,
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
    use crate::purging::start_periodic_purge;

    #[tokio::test]
    async fn when_the_purge_loop_runs_then_the_cache_deletes_expired_entries() {
        // Arrange
        let cache = Arc::new(RwLock::new(SpyCache::default()));

        // Act
        start_periodic_purge(cache.clone(), interval(Duration::from_secs(10000)));

        // Assert
        sleep(Duration::from_millis(1)).await;
        assert!(cache.write().await.purge_expired_called);
    }
}
