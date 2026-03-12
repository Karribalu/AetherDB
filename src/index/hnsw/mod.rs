//! HNSW (Hierarchical Navigable Small World) for approximate vector search.
//!
//! Per-segment HNSW graph built at flush time from the vector column.
//!
//! Distance metrics: cosine similarity, dot product, Euclidean (L2).
//!
//! Construction parameters (tunable per namespace):
//! - `M` — max connections per node per layer (default 16).
//! - `ef_construction` — beam width during index build (default 200).
//! - `ef_search` — beam width during query (default 64).
//!
//! # Reference implementation
//!
//! A brute-force linear scanner lives in `hnsw::reference`. Recall-at-k
//! is measured against it before the HNSW index is trusted in production.
//! Minimum required recall@10: 0.95.

// TODO (Week 15-18): implement HnswGraphBuilder, HnswGraphReader,
// brute-force reference, and recall-at-k measurement harness.
