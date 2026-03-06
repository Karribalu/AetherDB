# AetherDB AGENTS

This file defines the engineering law for AetherDB.

It is written for AI coding agents first and human contributors second. When there is ambiguity, optimize for preserving correctness, durability, and architectural integrity rather than delivering more surface area.

## Assumptions For This File

This document is based on the following confirmed or explicitly stated assumptions:

1. The file belongs at the repository root.
2. The tone is hard-rule oriented, not aspirational.
3. The scope is architecture and Tiger-style engineering rules only, not a product roadmap.
4. The current codebase is still in foundation mode.
5. Current module boundaries are `cli`, `core`, `engine`, and `storage`.

If any of these assumptions become false, update this file before expanding the codebase.

## Mission

Build AetherDB as a durable, AI-native database without sacrificing recoverability for velocity.

The project does not earn the right to call itself a database by parsing SQL, talking to a GPU, or demonstrating CXL emulation. It earns that right by surviving crashes, preserving invariants, and recovering deterministically.

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

1. Storage invariants.
2. Recovery semantics.
3. Data model correctness.
4. Query execution correctness.
5. Concurrency correctness.
6. Performance optimization.
7. Hardware-aware features such as GPU and CXL integration.

Do not reverse this order unless the repository owner explicitly changes the plan.

## Architecture Law

### Source Of Truth

For the MVP, the write-ahead log is the source of truth between snapshots.

Required rules:

1. A write is not committed until its WAL record is durable.
2. A snapshot is an optimization, not the primary authority.
3. Recovery always means: load the latest valid snapshot, then replay WAL entries after that boundary.
4. If a snapshot conflicts with a valid WAL tail, the WAL wins.

### Crash Semantics

Every persistence change must preserve these behaviors:

1. Partial WAL records are rejected or ignored through explicit framing and validation.
2. Partial snapshots are never accepted as valid current state.
3. Empty storage must boot cleanly.
4. Recovery must be deterministic and repeatable.
5. Restart behavior must be testable without manual inspection.

### Tiered Storage Intent

The storage model is conceptually:

1. Hot tier: in-memory working state.
2. Warm tier: memory-mapped persistence that simulates or later uses CXL-like characteristics.
3. Cold tier: durable snapshot state on disk.

Do not implement spill, migration, or placement heuristics before correctness exists for a single-tier reference path.

### Module Boundaries

Respect these responsibilities:

1. `cli`: process entrypoints, argument parsing, command dispatch.
2. `core`: shared configuration, error handling conventions, telemetry, cross-cutting runtime utilities.
3. `engine`: orchestration layer that composes storage, query, and runtime services.
4. `storage`: filesystem layout, WAL, snapshots, tier management, and persistence semantics.

Boundary rules:

1. `cli` must not embed storage logic.
2. `storage` must not depend on CLI concerns.
3. `engine` coordinates; it should not become a dumping ground for unrelated logic.
4. New modules must be added only when they reduce coupling or clarify responsibility.

## Feature Admission Rules

No feature may be added unless it satisfies all applicable admission rules.

### Persistence Features

Before merging WAL, snapshot, catalog, or spill behavior:

1. Define the failure model.
2. Define the on-disk format or record contract.
3. Add restart tests.
4. Add corruption or truncation tests where applicable.
5. State which component is authoritative during recovery.

### Query Features

Before expanding SQL support:

1. There must be a simpler in-process reference path for the same behavior when practical.
2. Semantics must be defined before optimization.
3. Query integration must not obscure storage invariants.

DataFusion is a tool, not the architecture. Do not let framework affordances dictate engine invariants.

### Vector Features

Before GPU or ANN acceleration:

1. Implement a correct CPU baseline.
2. Lock down vector storage semantics.
3. Define distance metric behavior exactly.
4. Prove result correctness against the baseline.

If accelerated results diverge from the CPU reference, the accelerated path is wrong until proven otherwise.

### AI Inference Features

Embedding or inference APIs are not core until the storage and recovery model is stable.

Do not add model execution features that obscure deterministic database behavior.

## Testing Law

Tests are part of the architecture, not follow-up work.

Minimum rules:

1. No persistence logic without restart tests.
2. No crash-sensitive path without truncation or partial-write tests.
3. No optimization without comparison to a correct baseline.
4. No storage tier transition without round-trip verification.
5. No concurrency feature without race-oriented validation.

Preferred test progression:

1. Unit tests for contracts and parsing.
2. Filesystem-backed tests for persistence and recovery.
3. Integration tests for end-to-end behavior.
4. Benchmarks only after correctness is stable.

## Performance Rules

Performance work is allowed only after the following are true:

1. There is a correct baseline implementation.
2. The bottleneck is measured.
3. The benchmark is reproducible.
4. The change does not weaken recovery or determinism.

Never use performance as justification to skip validation, remove invariants, or blur module boundaries.

## Operational Rules For Agents

When an AI agent works in this repository, it must:

1. Read the relevant architecture notes before expanding the design.
2. Preserve the current module boundaries unless there is a clear architectural reason to change them.
3. Prefer small, composable changes over sweeping rewrites.
4. Write down any new invariant before building on top of it.
5. Refuse to smuggle in hidden assumptions about durability, concurrency, or hardware.
6. Call out assumptions explicitly when requirements are ambiguous.
7. Avoid adding exciting features ahead of storage and recovery maturity.

An agent must not:

1. Introduce GPU, CXL, networking, or concurrency complexity to bypass unfinished core durability work.
2. Claim a component is production-ready without restart and corruption testing.
3. Treat generated framework behavior as a substitute for explicit database design.
4. Expand scope silently.

## Current State Constraints

At the time of writing, the repository is in Day 1 foundation mode.

Current observable facts:

1. The process initializes a local data directory layout.
2. Expected directories are `catalog`, `wal`, and `snapshots` beneath the root data directory.
3. The CLI exposes a minimal startup and info surface.
4. The next legitimate milestone is durable WAL plus recovery tests.

Do not behave as if table persistence, SQL execution, vector indexing, or server concurrency already exist.

## Change Control

When changing the architecture:

1. Update the relevant architecture note.
2. Update this file if the rule set or build order changes.
3. Keep claims aligned with the actual codebase state.

If code and documentation disagree, stop and reconcile them instead of building on top of the mismatch.
