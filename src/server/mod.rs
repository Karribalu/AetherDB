//! HTTP server surface.
//!
//! Built on `axum`. Wraps the engine — no direct storage or index access.
//!
//! Planned endpoints:
//! - `POST /v1/{namespace}/upsert` — ingest documents.
//! - `POST /v1/{namespace}/query` — JSON query.
//! - `POST /v1/{namespace}/sql` — SQL query.
//! - `DELETE /v1/{namespace}` — delete namespace.
//! - `GET /v1/{namespace}/schema` — describe namespace schema.
//! - `GET /health` — liveness check.
//!
//! The server is the last module added. It must not be implemented before the
//! write and read paths are correct and tested.

// TODO (Week 29): implement the axum router and handler functions.
