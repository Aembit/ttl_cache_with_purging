//! A time-to-live (TTL) cache implementation with optional background purging
//! for expired entries.
//!
//! ## Motivation
//!
//! We needed a caching implementation that would not return expired entries,
//! while also preventing expired entries from unnecessarily inflating the cache
//! size.
//!
//! ## Approach
//!
//! This TTL cache includes a background purge thread that will remove expired
//! cache entries on a specified interval. The purge thread uses `tokio` to take
//! advantage of its write-preferring `RwLock`.
//!
//! ## Example
//! ```rust
#![doc = include_str!("../examples/example.rs")]
//! ```
pub mod cache;
pub mod purging;
