//! S3-compatible object storage backend.
//!
//! Wraps the `object_store` crate to provide a narrow interface: put, get,
//! delete, and list. The rest of AetherDB never calls `object_store` directly.

// TODO (Week 6): implement ObjectStore abstraction over object_store crate.
