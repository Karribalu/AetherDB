//! Index structures: inverted index, BKD tree, bitmap, and HNSW.
//!
//! All index types are built from scratch in Rust. No Tantivy, FAISS, or
//! Lucene dependency. Each submodule:
//! 1. Defines a per-segment builder that produces a byte blob.
//! 2. Defines a reader that answers queries against that blob.
//! 3. Ships a naive brute-force reference implementation used in correctness
//!    tests to validate the optimized version.

pub mod bitmap;
pub mod bkd;
pub mod hnsw;
pub mod inverted;
