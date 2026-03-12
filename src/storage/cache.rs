//! NVMe local cache layer.
//!
//! Caches segment blobs on local NVMe disk to avoid repeated object storage
//! fetches. The cache is:
//!
//! - **Evictable**: segments are evicted LRU when the configured capacity is
//!   exceeded.
//! - **Reconstructible**: any evicted segment can be re-fetched from object
//!   storage. Cache loss is not data loss.
//! - **Content-addressed**: cached files are keyed by segment ID. A corrupted
//!   or missing cache file triggers a fetch, not an error.

// TODO (Week 7): implement NvmeCache with LRU eviction and async fetch-on-miss.
