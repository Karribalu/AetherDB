# AetherDB Roadmap

This roadmap assumes a single primary builder with disciplined weekly progress and a correctness-first approach.

## Planning Assumptions

- Duration: 30 weeks.
- Team shape: 1 builder full-time or near full-time.
- Goal: a release-quality single-node text and vector search database.
- Source of truth: object storage.
- Local acceleration tier: evictable NVMe cache.
- Query surfaces: SQL and JSON.
- Indexes built from scratch: inverted index, BKD tree, roaring bitmap, HNSW.
- Parser dependency allowed: `sqlparser` only as a parser, not as an execution model.

## Phase Overview

| Phase            | Weeks | Outcome                                                      |
| ---------------- | ----- | ------------------------------------------------------------ |
| Foundation       | 1-4   | Stable crate boundaries, binary segment format, test harness |
| Storage          | 5-8   | Local store, object store abstraction, NVMe cache, catalog   |
| Core Indexes I   | 9-14  | Inverted index, BKD tree, bitmap index with reference tests  |
| Core Indexes II  | 15-18 | HNSW plus recall validation                                  |
| Engine           | 19-22 | Write path and read path over immutable segments             |
| Query Layer      | 23-26 | SQL and JSON planners over shared logical plan               |
| Merge and Server | 27-30 | Background merge, HTTP API, stabilization                    |

## Weekly Milestones

### Week 1

- Freeze architecture and invariants.
- Define segment object model: namespace, schema, segment metadata, field metadata.
- Write the first implementation RFC for the segment binary format.
- Set up test categories: unit, integration, property tests, benchmarks.

### Week 2

- Define segment header and footer layout.
- Define checksums, offsets, versioning policy, and forward-compatibility rules.
- Define encoding primitives: varint, fixed-width little-endian, delta encoding.
- Add round-trip tests for primitive encoding.

### Week 3

- Implement codec layer for segment header/footer.
- Implement metadata serialization for schema and field descriptors.
- Define segment regions for column data and index blobs.
- Add corruption and partial-read tests.

### Week 4

- Implement first full segment round-trip test fixture.
- Lock initial segment format version as `v1alpha1`.
- Document invariants for partial writes and invalid checksum handling.
- Exit criterion: binary segment format is stable enough for storage work.

### Week 5

- Implement local filesystem segment store.
- Add write-read-delete tests for segment blobs.
- Define storage trait used by catalog and engine.
- Add local test utilities for fixture segments.

### Week 6

- Implement object storage abstraction on top of `object_store`.
- Support S3-compatible endpoints first.
- Add tests for put/get/list/delete semantics.
- Define durability contract for acknowledged writes.

### Week 7

- Implement NVMe cache metadata model.
- Implement fetch-on-miss behavior and cache population.
- Implement LRU eviction by byte budget.
- Add eviction and re-fetch correctness tests.

### Week 8

- Implement catalog manifest format and schema registry.
- Implement atomic catalog update protocol.
- Add tests for stale catalog writes and recovery from cache loss.
- Exit criterion: object storage plus cache plus catalog can safely describe active segments.

### Week 9

- Define tokenizer and term normalization rules.
- Implement naive reference full-text matcher.
- Implement postings list data structures.
- Add correctness tests for term lookup.

### Week 10

- Implement inverted index builder.
- Implement postings encoding and dictionary layout.
- Add doc frequency and term frequency validation tests.
- Start BM25 reference scorer.

### Week 11

- Implement inverted index reader and BM25 scorer.
- Add boolean query support over postings.
- Compare ranked results against reference scorer.
- Exit criterion: per-segment text search is correct.

### Week 12

- Implement naive numeric/range reference scanner.
- Define BKD node layout and split strategy.
- Build first BKD writer for integers and timestamps.
- Add range query correctness tests.

### Week 13

- Implement BKD reader.
- Extend BKD to floats and geo-ready point representation.
- Verify exact match and range semantics against reference scan.
- Exit criterion: numeric filtering is correct.

### Week 14

- Implement roaring bitmap containers and operations.
- Build bitmap index writer/reader for low-cardinality fields.
- Compare bitmap results against naive HashSet reference.
- Exit criterion: low-cardinality filters are correct.

### Week 15

- Implement brute-force vector search reference.
- Define HNSW graph serialization format.
- Implement distance metrics: cosine, dot product, L2.
- Add deterministic vector test fixtures.

### Week 16

- Implement HNSW graph builder.
- Implement insertion policy and layer assignment.
- Add construction invariants and graph integrity tests.
- Start recall measurement harness.

### Week 17

- Implement HNSW query path.
- Compare top-k recall against brute-force reference.
- Tune `M`, `ef_construction`, and `ef_search` defaults.
- Set minimum acceptable recall thresholds.

### Week 18

- Harden HNSW serialization and load path.
- Add larger synthetic dataset tests.
- Exit criterion: vector search is performant enough to continue and measurably correct.

### Week 19

- Define logical row model used during ingest.
- Implement write buffer and flush trigger rules.
- Implement segment assembly using codec plus indexes.
- Define commit protocol: object write before catalog publish.

### Week 20

- Implement write coordinator.
- Add namespace creation and schema validation on ingest.
- Add durability tests ensuring segment is visible before ack.
- Exit criterion: write path is correct.

### Week 21

- Define physical scan interfaces used by engine.
- Implement segment reader and candidate doc ID execution pipeline.
- Add row fetch and projection logic.
- Start merge/rank layer for multi-segment results.

### Week 22

- Implement read executor across segments.
- Support text, numeric, bitmap, and vector candidate generation.
- Implement final sort, top-k, and projection.
- Exit criterion: engine can execute logical plans across segment sets.

### Week 23

- Define shared logical plan AST.
- Map SQL `SELECT`, `WHERE`, `ORDER BY`, `LIMIT`, `OFFSET` onto it.
- Implement planner error handling and schema binding.
- Add SQL planner unit tests.

### Week 24

- Add full-text and vector SQL syntax mapping.
- Add plan equivalence tests for representative SQL queries.
- Exit criterion: SQL planner is usable for v0.1.

### Week 25

- Define JSON query schema.
- Implement JSON planner to the same logical plan AST.
- Add validation and error reporting for malformed filters.
- Add planner parity tests between SQL and JSON for equivalent queries.

### Week 26

- Add projections, pagination, and compound filters in JSON planner.
- Exit criterion: SQL and JSON both target the same engine cleanly.

### Week 27

- Implement merge policy and merge coordinator.
- Implement soft-delete handling for superseded segments.
- Add merge verification before old segment retirement.
- Add recovery tests for interrupted merges.

### Week 28

- Add background maintenance loops.
- Add telemetry, tracing, and engine counters.
- Run stabilization passes on storage and execution hot paths.
- Exit criterion: single-node engine is operationally coherent.

### Week 29

- Implement HTTP API with `axum`.
- Add endpoints for ingest, SQL query, JSON query, schema inspection, and health.
- Ensure server layer delegates only to engine.
- Add black-box API integration tests.

### Week 30

- Run end-to-end burn-in testing.
- Write operator documentation and configuration guide.
- Benchmark representative workloads.
- Tag first release candidate for `v0.1.0`.

## Release Gates

A phase is only complete when these conditions hold:

- All invariants for that phase are documented.
- Unit tests and integration tests for that phase pass.
- A naive reference implementation exists for every index introduced.
- No known correctness bug remains open in the completed phase.
- Public interfaces for that phase are stable enough for the next phase.

## Suggested Execution Discipline

- Reserve 1 day each week for test hardening and documentation.
- Do not overlap major subsystems until the current subsystem has explicit exit criteria met.
- Treat benchmarks as validation after correctness, not as a design driver.
- Keep one active milestone only; avoid parallel feature branches inside a single-person build.

## Earliest Useful Demo

A credible internal demo can happen by Week 22 if these are working:

- Ingest documents into immutable segments.
- Persist segments to object storage.
- Rehydrate segments through the NVMe cache.
- Execute text, numeric, bitmap, and vector retrieval over the engine.

That demo does not require HTTP or full SQL completeness yet.

## v0.1 Definition

`v0.1` is achieved when:

- Storage, cache, and catalog are correct.
- All four index types are implemented and reference-validated.
- The engine supports writes and reads over immutable segments.
- SQL and JSON planners both compile to the same logical plan.
- The HTTP API exposes ingest and query endpoints.
- End-to-end tests pass against local and object-backed storage.
