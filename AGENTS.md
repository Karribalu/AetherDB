# AetherDB AGENTS

This file defines the engineering law for AetherDB.

It is written for AI coding agents first and human contributors second. When there is ambiguity, optimize for preserving correctness, durability, and architectural integrity rather than delivering more surface area.

## What AetherDB Is

AetherDB is a **text and vector search database** built from scratch in Rust.


The system targets:

1. Full-text search with BM25 scoring (inverted index, built from scratch).
2. Vector similarity search (HNSW, built from scratch).
3. Numeric range queries (BKD tree, built from scratch).
4. Low-cardinality bitmap queries (Roaring bitmaps, built from scratch).
5. SQL query surface using `sqlparser` for parsing, custom planner.
6. JSON-based filter/search query surface.
7. Object storage as source of truth (S3-compatible).
8. NVMe local cache that is evictable and reconstructible.
9. Immutable segment architecture.

## Confirmed Assumptions

1. The file belongs at the repository root.
2. The design is inspired by turbopuffer and Lucene's segment model, but built independently.
3. All index structures are built from scratch in Rust. No Lucene, Tantivy, or FAISS.
4. The module structure is: `cli`, `core`, `catalog`, `storage`, `codec`, `index`, `query`, `engine`, `server`.
5. The repository is in foundation mode with crate/module scaffolding in place but no subsystem implementations yet.

## Tiger Style

Tiger Style means the system is built as if every weak shortcut will eventually become a production failure.

Non-negotiable principles:

1. Correctness before capability.
2. Durability before performance.
3. Simplicity before optimization.
4. Determinism before convenience.
5. Explicit invariants before feature growth.
6. Reference implementations before accelerated implementations.
7. Measurable behavior before marketing claims.

## Primary Build Order

All contributors and agents must respect this implementation order:

1. `codec` — binary segment format and encoding primitives.
2. `storage` — local file I/O for segments; then object storage abstraction.
3. NVMe cache layer inside `storage`.
4. `catalog` — segment manifest and schema registry on object storage.
5. `index::inverted` — per-segment full-text inverted index.
6. `index::bkd` — per-segment BKD tree for numeric/range queries.
7. `index::bitmap` — per-segment bitmap index.
8. `index::hnsw` — per-segment HNSW vector index.
9. `engine` write path — ingest, flush, segment creation, object storage commit.
10. `engine` read path — segment scan, index lookup, merge, rank.
11. `query::sql` — SQL → logical plan.
12. `query::json` — JSON filter → logical plan.
13. Merge coordinator.
14. `server` — HTTP API surface.

Do not reverse this order unless the repository owner explicitly changes the plan.

## Architecture Law

### Source Of Truth

Object storage is the source of truth. The catalog file in object storage is the authority for which segments exist.

Required rules:

1. A write is not committed until the segment is durable in object storage.
2. NVMe cache is subordinate to object storage. Cache loss is not data loss.
3. The catalog tracks the active segment set. It is updated atomically.
4. A segment not referenced in the catalog does not exist from the system's perspective.

### Segment Invariants

1. Segments are immutable once written to object storage.
2. A partial segment write is discarded, not partially read.
3. Index structures within a segment must be byte-reproducible from the same input data.
4. Merging produces new segments; old segments are soft-deleted after merge verification.

### Index Correctness

1. Every index type must have a naive reference implementation.
2. The optimized index must produce results identical to the reference implementation on the same data.
3. HNSW recall-at-k must be measured against brute-force before the index is trusted.
4. BM25 scores must be validated against a reference BM25 implementation.

### Module Boundaries

Respect these responsibilities:

1. `cli` — process entrypoints, argument parsing, command dispatch. No storage or index logic.
2. `core` — error types, configuration, telemetry, shared primitives.
3. `catalog` — schema registry, segment manifest, namespace metadata. Backed by object storage.
4. `storage` — object storage client abstraction, NVMe cache, segment byte-level read/write.
5. `codec` — binary encoding/decoding for segments and index structures.
6. `index` — inverted index, HNSW, BKD tree, bitmap index. Reads from `storage` and `codec`.
7. `query` — SQL planner, JSON planner, logical plan types. Does not touch storage directly.
8. `engine` — query execution, segment scanner, result merger, write coordinator. Coordinates `storage`, `index`, `query`.
9. `server` — HTTP/gRPC surface. Wraps `engine`. No direct storage or index access.

Boundary rules:

1. `cli` must not embed storage, index, or query logic.
2. `storage` must not depend on CLI, query, or index concerns.
3. `index` must not plan queries; it only answers index lookups.
4. `query` must not touch storage directly.
5. `engine` is the only coordinator. It must not become a dumping ground.
6. `server` wraps `engine`. That is its only job.

## Feature Admission Rules

No feature may be added unless it satisfies all applicable admission rules.

### Codec and Storage Features

Before merging segment format, storage backend, or cache behavior:

1. Define the on-disk or object-storage format contract.
2. Add round-trip serialization tests.
3. Add fetch-after-eviction tests for cache.
4. State the invariant for partial write handling.

### Index Features

Before merging any index implementation:

1. Implement a naive brute-force reference for the same query type.
2. Prove the index produces identical results to the reference on the same data.
3. Add correctness tests with generated and edge-case inputs.
4. For HNSW: measure and assert recall-at-k against brute force.

### Query Features

Before expanding SQL or JSON query support:

1. Define the logical plan the query maps onto.
2. Verify the plan executes correctly against the segment and index layer.
3. Query integration must not obscure storage or index invariants.

### Server Features

Before adding HTTP or gRPC surface:

1. The write path and read path must be correct and tested.
2. The server must not add business logic; it dispatches to `engine`.

## Testing Law

Tests are part of the architecture, not follow-up work.

Minimum rules:

1. No codec without round-trip serialization tests.
2. No storage without write-then-read and simulated eviction tests.
3. No index without correctness comparison to a reference implementation.
4. No HNSW without recall-at-k measurement.
5. No query planner without plan equivalence tests.
6. No write path without object-storage durability tests (segment visible before ack).
7. No concurrency feature without race-oriented validation.

Preferred test progression:

1. Unit tests for encoding contracts and data structure invariants.
2. File-backed tests for segment write/read correctness.
3. Object-storage mock tests for catalog and durability behavior.
4. Integration tests for end-to-end query behavior.
5. Benchmarks only after correctness is stable.

## Performance Rules

Performance work is allowed only after:

1. There is a correct reference implementation.
2. The bottleneck is measured.
3. The benchmark is reproducible.
4. The change does not weaken correctness or durability guarantees.

Never use performance as justification to skip validation or remove invariants.

## Operational Rules For Agents

When an AI agent works in this repository, it must:

1. Read Architecture.md before expanding the design.
2. Respect the build order. Do not implement query features before the codec and storage layer is correct.
3. Do not add GPU, distributed, or networking complexity before the single-node storage and index correctness is established.
4. Every index implementation must have a reference brute-force companion.
5. Call out assumptions explicitly when requirements are ambiguous.
6. Prefer small composable changes over sweeping rewrites.

An agent must not:

1. Use Tantivy, FAISS, or Lucene as a dependency. The indexes are built from scratch.
2. Claim an index is correct without a reference comparison test.
3. Add server or networking code before write/read path correctness is proven.
4. Treat `sqlparser` AST as the execution model. It is only the parse layer.
5. Expand scope silently.

## Current State

The repository now has the top-level crate structure and stub modules for `cli`, `core`, `catalog`, `storage`, `codec`, `index`, `query`, `engine`, and `server`.

Those modules define boundaries and implementation notes only. No storage engine, catalog protocol, index implementation, planner, or server behavior exists yet.

The next legitimate step is still defining the `codec` module concretely: segment header, binary layout, offsets, checksums, and round-trip tests.

Do not build anything else first.

## Change Control

When changing the architecture:

1. Update Architecture.md.
2. Update this file if the build order, module responsibilities, or invariants change.
3. Keep claims aligned with the actual codebase state.

If code and documentation disagree, stop and reconcile them instead of building on top of the mismatch.
