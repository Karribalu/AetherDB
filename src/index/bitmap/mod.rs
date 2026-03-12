//! Roaring bitmap index for low-cardinality fields.
//!
//! Used for: enum fields, boolean fields, tag sets, label filters.
//! Each unique value in a low-cardinality column gets its own roaring bitmap
//! of doc IDs. Filter queries use AND/OR/NOT on these bitmaps directly.
//!
//! Roaring bitmaps are built from scratch. Container types:
//! - Array container (< 4096 values): sorted u16 array.
//! - Bitset container (≥ 4096 values): 65536-bit bitset.
//! - Run-length encoded container (dense runs).
//!
//! # Reference implementation
//!
//! A naive HashSet-based filter lives in `bitmap::reference`.

// TODO (Week 14): implement RoaringBitmap, BitmapIndexBuilder, BitmapIndexReader,
// and the reference HashSet filter.
