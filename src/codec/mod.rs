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
//! │  Preamble     (fixed)    magic, version, header sizing   │
//! │  Header       (variable) open-time segment metadata      │
//! │  Body         (variable) column and index regions        │
//! │  Footer       (variable) region directory and metadata   │
//! │  Footer CRC32 (4 bytes)  crc32 of footer payload         │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # Encoding primitives
//!
//! - Variable-length integers (VarInt): unsigned 64-bit, group-varint encoded.
//! - Delta-encoded doc ID lists.
//! - Little-endian fixed-width integers for lengths and offsets.

//! The first concrete segment object model lives in `metadata` and defines the
//! namespace, schema, field, and segment descriptors that later header/footer
//! encoding will serialize.

pub mod metadata;

#[allow(unused_imports)]
pub use metadata::{
    FieldIndexKind, FieldMetadata, FieldStatistics, FieldType, FieldValue, NamespaceMetadata,
    SchemaFingerprint, SchemaMetadata, SegmentFieldMetadata, SegmentMetadata, VectorDistanceMetric,
};

// TODO (Phase 1 – Week 2-4): implement SegmentHeader, varint encode/decode,
// delta encoding, binary metadata serialization, and round-trip tests.
