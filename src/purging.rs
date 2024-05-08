//! Strategies for purging expired cache entries.
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use tokio::time::interval;

use crate::cache::Purgeable;

/// Kick-off a background task that will purge expired entries from the cache at the
/// specified interval.
pub fn start_periodic_purge<P>(cache: Arc<RwLock<P>>, purge_interval: Duration)
where
    P: Purgeable + Send + Sync + 'static,
{
    let mut purge_interval = interval(purge_interval);
    tokio::task::spawn(async move {
        loop {
            // Note that the first tick is instantaneous.
            purge_interval.tick().await;
            cache.write().unwrap().purge_expired();
        }
    });
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};
    use std::time::Duration;

    use tokio::time::sleep;

    use crate::cache::test_helpers::SpyCache;
    use crate::purging::start_periodic_purge;

    #[tokio::test]
    async fn when_the_purge_loop_runs_then_the_cache_deletes_expired_entries() {
        // Arrange
        let cache = Arc::new(RwLock::new(SpyCache::default()));

        // Act
        start_periodic_purge(cache.clone(), Duration::from_secs(10000));

        // Assert
        sleep(Duration::from_millis(10)).await;
        assert!(cache.write().unwrap().purge_expired_called);
    }
}
