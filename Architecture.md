# AetherDB Architecture

## Purpose

This document is the canonical architecture summary for AetherDB at its current stage.

It exists to make the system's invariants, build order, storage authority, and scope boundaries explicit in one place. If this file and the code disagree, stop and reconcile them before expanding the system.

## Current State

AetherDB is in foundation mode.

What exists today:

1. A Rust binary with a minimal CLI surface.
2. Initialization of a local data directory layout.
3. The expected storage directories beneath the data root:
   - `catalog/`
   - `wal/`
   - `snapshots/`

What does not exist yet:

1. Durable WAL records.
2. Snapshot creation and recovery.
3. Persistent table or catalog state.
4. SQL execution.
5. Vector indexing or search.
6. Networking.
7. GPU or CXL-aware execution paths.

This gap is intentional. The repository is being built in the order most likely to produce a durable system rather than a fragile demo.

## Mission

Build AetherDB as a durable, AI-native database without sacrificing recoverability for velocity.

The system does not earn the right to call itself a database by parsing SQL, exposing AI features, or demonstrating accelerated execution. It earns that right by surviving crashes, preserving invariants, and recovering deterministically.

## Design Principles

The architecture follows Tiger Style rules:

1. Correctness before capability.
2. Durability before performance.
3. Simplicity before optimization.
4. Determinism before convenience.
5. Explicit invariants before feature growth.
6. Reference implementations before accelerated implementations.
7. Measurable behavior before performance or marketing claims.

These are architectural constraints, not preferences.

## Implementation Order

All work should follow this sequence:

1. Storage invariants.
2. Recovery semantics.
3. Data model correctness.
4. Query execution correctness.
5. Concurrency correctness.
6. Performance optimization.
7. Hardware-aware features such as GPU and CXL integration.

If a proposed feature tries to skip this order, it is likely arriving too early.

## System Model

The intended system shape is a single-node database with a staged storage hierarchy and explicit recovery semantics.

The architecture is organized around three ideas:

1. The WAL is the authority for committed state between snapshots.
2. Recovery must be deterministic and repeatable.
3. Higher-level features such as SQL, vector search, networking, and acceleration are downstream of a correct storage core.

## Source Of Truth

For the MVP, the write-ahead log is the source of truth between snapshots.

Rules:

1. A write is not committed until its WAL record is durable.
2. A snapshot is an optimization, not the primary authority.
3. Recovery always means loading the latest valid snapshot and replaying WAL entries after that boundary.
4. If a snapshot conflicts with a valid WAL tail, the WAL wins.

This means the system is not defined by "WAL or snapshot" as competing authorities. The authority model is:

1. Snapshot for recovery acceleration.
2. WAL for committed truth between snapshot boundaries.

## Storage Tiers

The tier model is intentional but still early.

### Hot Tier

The hot tier is the in-memory working state used by the engine for active operations.

Current expectation:

1. Use simple reference structures.
2. Favor clarity and correctness over specialized layouts.
3. Do not optimize hot-tier structures in ways that obscure persistence semantics.

### Warm Tier

The warm tier is memory-mapped persistence intended to simulate, and later possibly use, CXL-like characteristics.

Current constraint:

1. Warm-tier design must not bypass WAL and recovery correctness.
2. No spill policies, migration heuristics, or placement logic should land before the single-tier reference path is correct.

### Cold Tier

The cold tier is durable snapshot state on disk.

Current expectation:

1. Snapshots represent materialized state at a known boundary.
2. A partial or invalid snapshot must never be accepted as current state.
3. Snapshot format and validation rules must be explicit before snapshotting is considered complete.

## Write Path

The repository has defined the commit rule even though the full write path is not implemented yet.

The intended reference write path is:

1. Accept and validate the logical write request.
2. Encode a WAL record using explicit framing and validation metadata.
3. Append the WAL record.
4. Durably flush the WAL record.
5. Only then acknowledge commit.
6. Reflect the committed change in in-memory working state.
7. Materialize snapshots later as a separate recovery optimization.

Required invariant:

An acknowledged write must exist durably in the WAL.

Anything that acknowledges a write before WAL durability violates the architecture.

## Crash Semantics

Persistence work must preserve these behaviors:

1. Partial WAL records are ignored or rejected through explicit framing and validation.
2. Partial snapshots are never accepted as valid current state.
3. Empty storage boots as a clean database.
4. Recovery is deterministic and repeatable.
5. Restart behavior is testable without manual inspection.

Operationally, this implies:

1. If a crash happens before WAL durability, the write is not committed.
2. If a crash happens after WAL durability but before snapshotting, recovery must replay the WAL.
3. Truncated or corrupt WAL tails must not be interpreted as valid committed state.

## Recovery Path

Startup recovery should follow a fixed sequence:

1. Detect whether storage is empty.
2. If empty, boot cleanly with an empty database state.
3. If a snapshot exists, load the latest valid snapshot.
4. Determine the replay boundary from that snapshot.
5. Replay WAL records after the snapshot boundary.
6. Reject or ignore invalid partial WAL records according to the WAL validation rules.
7. Publish the recovered state only after replay completes successfully.

Recovery must be deterministic. Running recovery multiple times over the same on-disk state must produce the same result.

## Module Boundaries

The current module responsibilities are:

1. `cli`: process entrypoints, argument parsing, and command dispatch.
2. `core`: shared configuration, error handling conventions, telemetry, and cross-cutting runtime utilities.
3. `engine`: orchestration layer that composes storage, query, and runtime services.
4. `storage`: filesystem layout, WAL, snapshots, tier management, and persistence semantics.

Boundary rules:

1. `cli` must not embed storage logic.
2. `storage` must not depend on CLI concerns.
3. `engine` coordinates; it must not become a dumping ground for unrelated logic.
4. New modules should exist only when they reduce coupling or clarify responsibility.

## Feature Admission Rules

No new feature should land unless its architectural prerequisites are satisfied.

### Persistence Features

Before merging WAL, snapshot, catalog, or spill behavior:

1. Define the failure model.
2. Define the on-disk format or record contract.
3. Add restart tests.
4. Add corruption or truncation tests where applicable.
5. State which component is authoritative during recovery.

### Query Features

Before expanding SQL support:

1. There must be a simpler in-process reference path when practical.
2. Semantics must be defined before optimization.
3. Query integration must not obscure storage invariants.

### Vector Features

Before ANN, GPU, or other acceleration paths:

1. Implement a correct CPU baseline.
2. Lock down vector storage semantics.
3. Define distance metric behavior exactly.
4. Prove result correctness against the baseline.

### AI Features

Embedding or inference APIs are not core until storage and recovery are stable.

## Testing Requirements

Tests are part of the architecture.

Minimum rules:

1. No persistence logic without restart tests.
2. No crash-sensitive path without truncation or partial-write tests.
3. No optimization without comparison to a correct baseline.
4. No storage tier transition without round-trip verification.
5. No concurrency feature without race-oriented validation.

Preferred progression:

1. Unit tests for contracts and parsing.
2. Filesystem-backed tests for persistence and recovery.
3. Integration tests for end-to-end behavior.
4. Benchmarks only after correctness is stable.

## Explicit Non-Goals Until v0.1

Until the storage and recovery core is trustworthy, the following are explicitly out of scope:

1. SQL execution.
2. Persistent table metadata beyond the current directory layout.
3. Networking and server behavior.
4. Vector indexing and ANN search.
5. GPU acceleration.
6. CXL-aware data placement or execution.
7. AI inference or embedding execution APIs.
8. Concurrency features that arrive before single-threaded correctness.
9. Performance tuning that weakens determinism or durability.
10. Spill, migration, or placement heuristics across hot, warm, and cold tiers.

## Legitimate v0.1 Target

The next legitimate milestone is not broader capability. It is a trustworthy storage baseline.

v0.1 should mean:

1. A concrete WAL record format exists.
2. Writes are committed only after WAL durability.
3. Crash and restart behavior is specified and tested.
4. Recovery from empty storage, valid WAL-only state, and snapshot-plus-WAL state is deterministic.
5. Corrupt or partial WAL and snapshot cases are rejected safely.

If those properties are not true, higher-level features are premature.

## Change Control

When the architecture changes:

1. Update this document.
2. Update any supporting architecture notes and agent instructions that define the same invariants.
3. Keep claims aligned with the actual codebase state.

If code and documentation disagree, documentation is not a license to continue. Reconcile the mismatch first.
