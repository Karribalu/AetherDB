# AetherDB Day 1 Foundation

## Goal

Day 1 establishes the smallest executable shape of AetherDB without pretending that the database already exists. The code is intentionally narrow: start the process, prepare the on-disk layout, and make the first storage contract explicit.

## Tiger Rules

1. Correctness before capability.
2. Durability before performance.
3. A simpler reference path must exist before any optimization.
4. Crash behavior must be explained before persistence code lands.
5. Every new storage feature must be restart-tested.

## First Runtime Contract

The process may initialize only these paths beneath the data directory:

- `catalog/` for schema metadata.
- `wal/` for append-only recovery records.
- `snapshots/` for point-in-time materialized state.

Creating the layout is the only side effect allowed on Day 1.

## Source Of Truth

For the MVP, the write-ahead log is the source of truth between snapshots.

Rules:

1. An acknowledged write must exist in the WAL before it is considered committed.
2. A snapshot is an optimization for faster recovery, not the primary source of truth.
3. Recovery always means: load latest valid snapshot, then replay WAL records after the snapshot boundary.

## Crash Semantics

These behaviors are required before any persistent write path ships:

1. Partial WAL record: ignored or rejected with checksum/length validation.
2. Partial snapshot: never accepted as current state.
3. Missing snapshot with valid WAL: recover from WAL.
4. Empty data directory: boot as a clean database.

## Planned Module Boundaries

- `cli`: argument parsing and process entrypoints.
- `core`: shared config and telemetry.
- `engine`: orchestration of storage, query, and runtime services.
- `storage`: filesystem layout, WAL, snapshots, and tier management.

## Explicit Non-Goals For Day 1

Day 1 does not implement:

- SQL execution.
- Table metadata persistence.
- WAL records.
- Snapshots.
- Networking.
- Vector search.
- CXL or GPU integration.

## Day 2 Entry Point

Next work should add a concrete WAL record format and tests for restart recovery under normal shutdown and simulated crash conditions.
