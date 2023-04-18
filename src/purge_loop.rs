use std::{hash::Hash, sync::Arc};

use tokio::{sync::RwLock, time::Interval};

use crate::cache::Cacheable;

pub fn start_purge_loop<C, K, V>(cache: Arc<RwLock<C>>, mut purge_interval: Interval)
where
    C: Cacheable<K, V> + Send + Sync + 'static,
    K: Eq + Hash + Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    tokio::task::spawn(async move {
        loop {
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

    use crate::cache::test_helpers::MockCache;
    use crate::purge_loop::start_purge_loop;

    #[tokio::test]
    async fn when_the_purge_loop_runs_then_the_cache_deletes_expired_entries() {
        // Arrange
        let mut cache = MockCache::<&str, &str>::new();
        cache.expect_purge_expired().times(1);
        let cache = Arc::new(RwLock::new(cache));

        // Act
        start_purge_loop(cache.clone(), interval(Duration::from_secs(10000)));

        // Assert - expectation in mock
        sleep(Duration::from_millis(1)).await;
        cache.write().await.checkpoint();
    }
}
