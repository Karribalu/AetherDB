//! Inverted index for full-text search with BM25 scoring.
//!
//! Per-segment build process:
//! 1. Tokenize text fields → term stream.
//! 2. Build term → postings list (sorted doc IDs + term frequencies).
//! 3. Delta-encode doc IDs; varint-encode frequencies.
//! 4. Write postings block + term dictionary to the segment blob region.
//!
//! Queries supported: term, phrase, prefix, boolean (AND/OR/NOT).
//! Scoring: BM25 (k1=1.2, b=0.75 defaults, tunable per namespace).
//!
//! # Reference implementation
//!
//! A naive linear-scan BM25 scorer lives in `inverted::reference`. The
//! optimised index must produce identical ranked results on the same data.

// TODO (Week 9-11): implement InvertedIndexBuilder, InvertedIndexReader,
// BM25Scorer, and the brute-force reference scorer.
