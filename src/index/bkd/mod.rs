//! BKD tree (block k-d tree) for numeric and range queries.
//!
//! Handles: integers, floats, timestamps, and geo-coordinates.
//! Supports: point lookup, range queries, geo-bounding-box queries.
//!
//! The on-disk layout follows Lucene's BKD paper (Zavist et al., 2016) but
//! is re-implemented independently in safe Rust.
//!
//! # Reference implementation
//!
//! A naive linear-scan range matcher lives in `bkd::reference`.
//! The BKD tree must return identical doc ID sets on the same data.

// TODO (Week 12-13): implement BkdBuilder, BkdReader, and the brute-force
// reference scanner.
