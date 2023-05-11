# ttl-cache

A time-to-live (TTL) cache implementation with optional background purging
for expired entries.

## Motivation

We needed a caching implementation that would not return expired entries,
while also preventing expired entries from unnecessarily inflating the cache
size.

## Approach

This TTL cache includes a background purge thread that will remove expired
cache entries on a specified interval. The purge thread uses `tokio` to take
advantage of its write-preferring `RwLock`.

## Example

```rust
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::{sync::RwLock, time::interval};
use ttl_cache::{cache::TtlCache, purging::start_periodic_purge};

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
```

## License

Licensed under either of

* Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Code of Conduct

All behavior is governed by the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).
