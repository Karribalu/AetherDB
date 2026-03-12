//! JSON query planner.
//!
//! Accepts a JSON object describing a query (similar to turbopuffer's API)
//! and maps it onto a `LogicalPlan`. Shares the same execution path as
//! the SQL planner — only the front-end parse step differs.
//!
//! Example query shape:
//! ```json
//! {
//!   "namespace": "articles",
//!   "vector": [0.1, 0.4, ...],
//!   "distance_metric": "cosine",
//!   "top_k": 10,
//!   "filters": {
//!     "and": [
//!       { "field": "status", "eq": "published" },
//!       { "field": "published_at", "gte": "2024-01-01" }
//!     ]
//!   },
//!   "return_fields": ["id", "title", "published_at"]
//! }
//! ```

// TODO (Week 25-26): implement JsonQueryPlanner.
