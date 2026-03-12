//! Storage layer: object storage abstraction + NVMe local cache.
//!
//! Responsibilities:
//! - Byte-level read/write of segment blobs.
//! - Object storage client abstraction (S3-compatible, GCS, Azure, local).
//! - NVMe cache: LRU segment cache backed by local disk.
//!
//! Boundary rules:
//! - This module has no knowledge of index structures or query plans.
//! - All callers receive or supply raw `bytes::Bytes`.
//! - The `catalog` module uses this module to persist the catalog blob.

pub mod cache;
pub mod local;
pub mod object;

// TODO (Phase 1 – Week 5-7): implement ObjectStore trait, LocalStore,
// S3Store (via object_store crate), and NvmeCache.
