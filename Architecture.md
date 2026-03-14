# AetherDB Architecture

## Purpose

This document is the canonical architecture reference for AetherDB.

It defines the system model, module boundaries, build order, and invariants. If this file and the code disagree, reconcile them before expanding the system.

---

## What AetherDB Is

AetherDB is a **text and vector search database** built from scratch in Rust.

Core design targets:

1. **Object storage as primary persistence** — all segment data is durable in object storage (S3-compatible). There is no WAL on local disk. Object storage is the source of truth.
2. **NVMe-backed local cache** — a local NVMe tier caches hot segments. Cache is evictable and reconstructible from object storage at any time. Cache loss is not data loss.
3. **Unlimited scale** — because object storage is the primary tier, the dataset size is bounded only by object storage capacity, not local disk.
4. **Dual query surface** — SQL queries and JSON-based filter/search queries over the same underlying segments.
5. **Multiple index types** — inverted index for full-text search, HNSW for vector similarity, BKD-tree for numeric range, and bitmap indexes for low-cardinality fields. All built from scratch in Rust, inspired by Lucene's index designs but not using Lucene.
6. **Immutable segment architecture** — data is written as immutable segments. Mutations produce new segments. Segments are merged in the background. This is the same model used by Lucene, RocksDB, and turbopuffer.

This database is **not** a general-purpose OLTP database. It targets analytical reads, full-text search, and vector similarity workloads.

---

## System Model

```
Client
  │
  ▼
Query Layer  ──────────────────────────────────────────────────┐
  │  SQL parser + logical planner + JSON query planner         │
  │                                                            │
  ▼                                                            │
Execution Engine                                               │
  │  Segment scanner, index reader, merge coordinator          │
  │                                                            │
  ▼                                                            │
Index Layer                                                    │
  │  Inverted index, HNSW (vector), BKD tree (numeric),        │
  │  Bitmap index (low-cardinality)                            │
  │                                                            │
  ▼                                                            │
Segment Store                                                  │
  │  Immutable columnar segments (custom binary format)        │
  │                                                            │
  ├──► NVMe Cache Tier   (hot segments, evictable)             │
  │                                                            │
  └──► Object Storage    (all segments, source of truth)       │
```

---

## Storage Model

### Immutable Segments

Every write operation produces a new immutable segment. A segment contains:

1. Columnar row data encoded in a compact binary format (defined by AetherDB, not Parquet or Arrow — those may be used as read targets later).
2. An inverted index for text fields.
3. An HNSW graph for vector fields.
4. A BKD tree for numeric and geo fields.
5. A bitmap index for low-cardinality fields such as labels and enums.
6. Metadata: segment ID, schema fingerprint, min/max field values, row count, byte size, object storage key.

A segment is immutable once written. It is never modified in place.

### Segment Object Model

The first concrete metadata model for segments is split into four concepts:

1. **Namespace metadata** identifies the logical collection exposed to clients and anchors the active schema.
2. **Schema metadata** is versioned and fingerprinted. It contains an ordered list of field descriptors with stable field IDs.
3. **Field metadata** defines field name, logical type, nullability, storage behavior, and declared index kinds.
4. **Segment metadata** binds an immutable segment to one namespace and one exact schema fingerprint, and records row count, byte size, object storage key, and per-field statistics.

This model is intentionally limited to durable storage facts. Query planning, execution policy, and cache state are not part of the segment object model.

### Object Storage as Source of Truth

1. All committed segments must be durably written to object storage before a write is acknowledged.
2. NVMe local cache holds warm copies of recent or frequently accessed segments.
3. Cache eviction is safe at any time. On cache miss, the segment is fetched from object storage.
4. A catalog file in object storage tracks the currently active set of segment keys and schema state.
5. The catalog is updated atomically using object storage conditional writes (compare-and-swap on version/etag where available, otherwise append-only catalog log).

### Segment Lifecycle

```
ingest → write buffer → flush → segment (object storage + optional NVMe cache)
                                        │
                          background merge → larger segment
                                        │
                          soft delete old segments after merge is verified
```

### NVMe Cache Tier

1. Segments are cached in full on local NVMe on first access or at write time.
2. The cache index maps segment ID → local path + size.
3. Eviction policy: LRU by segment last-access time, capped by configured byte budget.
4. Cache is always reconstructible from object storage. Cache corruption or loss is handled by fetching the segment again.

---

## Index Types

All indexes are built from scratch in Rust. No Lucene, Tantivy, or FAISS dependency.

### Inverted Index (Full-Text)

- Tokenize text fields into terms.
- Build term → postings list (sorted doc IDs + term frequencies).
- Support BM25 scoring.
- Store per-segment on object storage alongside segment data.
- Postings lists use delta-encoding and variable-length integer encoding.
- Support phrase queries, prefix queries, and boolean queries.

### HNSW (Vector Similarity)

- Hierarchical Navigable Small World graph for approximate nearest neighbor search.
- Per-segment HNSW graph built at flush time from the vector column.
- Support cosine, dot product, and Euclidean distance metrics.
- Results from per-segment HNSW are merged and re-ranked at query time.
- CPU baseline first; no GPU acceleration until correctness is proven.

### BKD Tree (Numeric / Range)

- Multi-dimensional k-d tree variant for numeric range and spatial queries.
- Used for integer, float, timestamp, and geo-coordinate fields.
- Per-segment block KD-tree stored in compact binary form.

### Bitmap Index (Low-Cardinality)

- Roaring bitmap per value per field for low-cardinality fields (enums, tags, boolean).
- Direct AND/OR operations for filter queries.

---

## Query Layer

### SQL Query Surface

- Use `sqlparser` crate to parse SQL into an AST.
- Build a custom logical planner that maps SQL constructs onto segment operations.
- The SQL planner does not replace the storage model; it maps onto it.
- Supported initially: `SELECT`, `WHERE`, `ORDER BY`, `LIMIT`, `OFFSET`.
- Full-text predicates via `MATCH()` or `WHERE text_field ~~ 'query'` syntax.
- Vector predicates via `ORDER BY vector_distance(field, [...]) LIMIT k`.

### JSON Query Surface

- A JSON-based filter and search API similar to turbopuffer's query format.
- Queries are plain JSON objects describing filter conditions, vector search, and field projections.
- The JSON planner operates on the same logical plan representation as the SQL planner.
- Both surfaces share the same execution engine.

### Execution

1. Parse query (SQL AST or JSON plan).
2. Resolve schema and identify relevant segments.
3. Per-segment: apply index predicates (inverted, BKD, bitmap) to get candidate doc IDs.
4. Per-segment: fetch and decode matching rows from columnar data.
5. Merge results across segments.
6. Apply ranking, scoring, and sorting.
7. Return projected result set.

---

## Module Boundaries

| Module    | Responsibility                                                     |
| --------- | ------------------------------------------------------------------ |
| `cli`     | Process entrypoints, argument parsing, command dispatch            |
| `core`    | Error types, configuration, telemetry, shared primitives           |
| `catalog` | Schema registry, segment manifest, namespace metadata              |
| `storage` | Object storage client abstraction, NVMe cache, segment read/write  |
| `index`   | Inverted index, HNSW, BKD tree, bitmap index implementations       |
| `codec`   | Binary encoding/decoding for segments and index structures         |
| `query`   | SQL planner, JSON planner, logical plan types, optimizer           |
| `engine`  | Query execution, segment scanner, result merger, write coordinator |
| `server`  | HTTP/gRPC server surface (post-correctness milestone)              |

Module boundary rules:

1. `cli` has no storage, index, or query logic.
2. `storage` has no query or index logic. It provides byte-level read/write over segments.
3. `index` reads from `storage` and `codec`. It does not plan queries.
4. `query` does not touch storage directly. It produces logical plans consumed by `engine`.
5. `engine` is the only layer allowed to coordinate across `storage`, `index`, and `query`.
6. `server` wraps `engine`. It has no direct storage or index access.

---

## Build Order

This is the mandatory sequence. Do not jump ahead.

1. **Codec** — define the binary segment format and encoding primitives.
2. **Storage** — local file I/O for segments, then object storage client abstraction.
3. **NVMe cache** — cache layer over storage with eviction policy.
4. **Catalog** — segment manifest and schema registry backed by object storage.
5. **Index: Inverted Index** — per-segment full-text index.
6. **Index: BKD Tree** — per-segment numeric range index.
7. **Index: Bitmap** — per-segment bitmap index for low-cardinality fields.
8. **Index: HNSW** — per-segment vector index.
9. **Engine: Write path** — ingest, flush, segment creation.
10. **Engine: Read path** — segment scan, index lookup, merge, rank.
11. **Query: SQL planner** — SQL → logical plan.
12. **Query: JSON planner** — JSON filter → logical plan.
13. **Merge coordinator** — background segment merging.
14. **Server** — HTTP API surface.

---

## Correctness Rules

1. A write is not acknowledged until the segment is durable in object storage.
2. A segment on NVMe cache that cannot be verified against the catalog is not used.
3. Partial segment writes are discarded, not partially read.
4. The catalog is the authority for which segments exist. Local cache is subordinate.
5. Index structures within a segment must be byte-reproducible from the same input data.
6. HNSW results must be verified against a brute-force CPU baseline before the index is trusted.

---

## Testing Requirements

1. No codec without round-trip serialization tests.
2. No storage layer without write-then-read verification and simulated fetch-after-eviction tests.
3. No index structure without correctness comparison to a naive reference implementation.
4. No HNSW without recall-at-k measurement against brute force.
5. No query planner without plan equivalence tests.
6. No write path without durability tests (segment visible in object storage before ack).
7. Benchmarks only after correctness is stable.

---

## Explicit Non-Goals for v0.1

1. Distributed multi-node coordination.
2. OLTP transactions or row-level locking.
3. GPU-accelerated vector search.
4. Replication.
5. Authentication and authorization.
6. Query optimizer cost estimation.
7. Streaming ingestion.

---

## v0.1 Target

v0.1 is complete when:

1. Binary segment format is defined, encoded, and round-trip tested.
2. A segment can be written to and read from local disk (object storage client abstracted).
3. Inverted index is built per-segment and queried correctly.
4. A simple SQL query (`SELECT ... WHERE ...`) executes against an in-process segment store.
5. A JSON filter query executes over the same segments.
6. All correctness tests pass.

---

## Change Control

When the architecture changes:

1. Update this document first.
2. Update AGENTS.md if build order or module boundaries change.
3. Keep claims aligned with actual codebase state.
4. If code and documentation disagree, reconcile before building further.
