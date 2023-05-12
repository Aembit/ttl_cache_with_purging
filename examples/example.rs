use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::{sync::RwLock, time::interval};
use ttl_cache_with_purging::{cache::TtlCache, purging::start_periodic_purge};

const MIN_IN_SECS: u64 = 60;
const HOUR_IN_SECS: u64 = 60 * MIN_IN_SECS;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Cache setup
    let cache = Arc::new(RwLock::new(TtlCache::new()));
    let purge_interval = interval(Duration::from_secs(MIN_IN_SECS));
    start_periodic_purge(cache.clone(), purge_interval);

    // Add entries
    let key = "key1";
    let val = "val1";

    let expires_at = Instant::now()
        .checked_add(Duration::from_secs(HOUR_IN_SECS))
        .unwrap();
    cache.write().await.insert(key, val, expires_at);

    // Read entries
    let _cached_val = cache.read().await.get(key).unwrap();
    let (_cached_val, _expires_at) = cache.read().await.get_value_and_expiration(key).unwrap();
}
