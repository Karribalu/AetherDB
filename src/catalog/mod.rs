//! Catalog: segment manifest and schema registry.
//!
//! The catalog is the authority for what exists in AetherDB:
//! - Which segments are active (keys, sizes, schema fingerprints).
//! - The schema for each namespace (field names, types, index config).
//!
//! The catalog is stored as a JSON blob in object storage at a well-known key.
//! It is updated atomically using conditional writes (check-and-set).
//!
//! Local cache holds a copy; it is always subordinate to the object storage
//! version. On conflict the object storage version wins.

// TODO (Week 8): implement CatalogEntry, SchemaDef, SegmentMeta, and the
// compare-and-swap catalog update protocol.
