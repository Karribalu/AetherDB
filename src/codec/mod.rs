//! Binary segment format: encoding and decoding primitives.
//!
//! Every on-disk and object-storage byte layout is defined here.
//! Nothing outside `codec` should write raw bytes. All callers go through
//! the encode/decode functions in this module.
//!
//! # Segment file layout
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │  Magic bytes  (8 bytes)  "AETHERDB"                     │
//! │  Version      (1 byte)   format version                 │
//! │  Header       (variable) SegmentHeader encoded as bytes  │
//! │  Column data  (variable) columnar row data               │
//! │  Inverted idx (variable) inverted index postings         │
//! │  BKD tree     (variable) numeric range index             │
//! │  Bitmap idx   (variable) low-cardinality field bitmaps   │
//! │  HNSW graph   (variable) vector similarity index         │
//! │  Footer CRC32 (4 bytes)  crc32 of everything above      │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # Encoding primitives
//!
//! - Variable-length integers (VarInt): unsigned 64-bit, group-varint encoded.
//! - Delta-encoded doc ID lists.
//! - Little-endian fixed-width integers for lengths and offsets.

// TODO (Phase 1 – Week 3-4): implement SegmentHeader, varint encode/decode,
// delta encoding, and round-trip tests.
