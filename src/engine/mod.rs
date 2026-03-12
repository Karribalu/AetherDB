//! Execution engine: the single coordinator across storage, index, and query.
//!
//! Responsibilities:
//! - Write path: accept ingest, buffer rows, flush to a segment, commit to
//!   object storage, update catalog.
//! - Read path: resolve query to segments, run per-segment index lookups,
//!   fetch and decode matching rows, merge and rank results.
//! - Merge coordinator: background task that merges small segments into
//!   larger ones and soft-deletes the source segments post-verification.
//!
//! Boundary rules:
//! - The engine is the only layer allowed to coordinate across storage,
//!   index, and query.
//! - It does not expose raw storage or index types to callers; it answers
//!   typed result sets.

pub mod read;
pub mod write;

// TODO (Week 19-22): implement EngineHandle, WriteCoordinator, ReadExecutor.
