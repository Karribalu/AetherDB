# AetherDB

AetherDB is an AI-native database being built with Tiger Style discipline: durability first, correctness first, and no shortcuts that make great demos but weak systems.

The long-term vision is to build a database that feels native to the next hardware and AI era while still behaving like a serious database under failure. That means relational data, vector data, tiered memory, accelerated search, and in-database AI capabilities, but only after the storage and recovery core has earned trust.

## Assumptions Behind This README

This README is written using the following assumptions:

1. AetherDB is intended to become a single-node, AI-native database before it grows into anything larger.
2. The project values Tiger Style engineering more than shipping flashy features early.
3. The near-term goal is a real MVP, not a prototype that collapses under crash or restart.
4. The current repository is still in foundation mode and has not yet implemented the durable segment codec, storage layers, or recovery behavior around immutable segments.

If any of these assumptions change, this README should change with them.

## Vision

The target is a database that combines four worlds that are usually built separately:

1. Relational query execution for structured application data.
2. Native vector storage and search for AI retrieval workloads.
3. Tiered memory and storage that can grow from RAM to memory-mapped warm tiers to durable cold state.
4. Hardware-aware acceleration paths for GPU and CXL-era systems.

The point is not novelty for its own sake. The point is to make a database that can serve traditional application data and AI-era workloads in one coherent engine.

## What AetherDB Is Trying To Become

AetherDB is aiming toward the following end state:

1. SQL-first access for relational and vector-aware workloads.
2. Tables that can store both standard columns and native vector columns.
3. A hot, warm, and cold storage model:
   RAM for active working state.
   NVMe-backed warm cache for large working sets.
   Object-storage-backed cold tier as the source of truth.
4. CPU-correct reference implementations first, with GPU acceleration later.
5. Durable persistence through immutable segments stored in object storage.
6. A simple server and CLI surface that stays understandable while the engine matures.

## Tiger Style Philosophy

AetherDB does not get to call itself a database because it can parse SQL or run vector search on a GPU. It earns that claim only if it survives crashes, recovers deterministically, preserves invariants, and can explain exactly why it is correct.

Tiger Style in this repository means:

1. Correctness before capability.
2. Durability before performance.
3. Simplicity before optimization.
4. Determinism before convenience.
5. Explicit invariants before feature growth.
6. Reference implementations before accelerated implementations.
7. Measured behavior before bold claims.

## Architecture Direction

The architectural direction for AetherDB is intentionally staged.

1. Object storage is the source of truth for committed segments.
2. Recovery must be deterministic from the catalog plus segment objects.
3. NVMe cache is an optimization, not the primary authority.
4. Warm-tier and hardware-aware ideas must not bypass storage correctness.
5. SQL, vector search, networking, and acceleration all sit on top of a reliable storage core, not the other way around.

Current module boundaries:

1. `cli` for command entrypoints.
2. `core` for shared configuration and telemetry.
3. `catalog` for namespace and segment manifests.
4. `storage` for object storage and NVMe cache semantics.
5. `codec` for segment binary format and metadata encoding.
6. `engine` for orchestration.

## Why This Exists

Most systems force a choice between being a serious database and being AI-friendly. AetherDB is an attempt to close that gap without sacrificing engineering discipline.

The project is being built around this idea:

1. Structured data and embedding data should not live in totally separate worlds by default.
2. The memory hierarchy matters and will matter more as CXL-style systems become practical.
3. GPU acceleration is valuable, but only if it sits behind a correct CPU baseline.
4. AI-native infrastructure still needs boring, dependable database recovery semantics.

## Current Status

The repository is in Day 1 foundation mode.

What exists today:

1. A Rust crate with a minimal CLI.
2. Crate boundaries for `cli`, `core`, `catalog`, `storage`, `codec`, `index`, `query`, `engine`, and `server`.
3. Architecture rules captured in [AGENTS.md](/Users/balasubramanyam/open-source/aetherdb/AGENTS.md) and [Architecture.md](/Users/balasubramanyam/open-source/aetherdb/Architecture.md).

What does not exist yet:

1. Durable segment header and footer encoding.
2. Local and object-backed segment storage.
3. Catalog update protocol.
4. Index implementations.
5. Query execution.
6. HTTP serving.
7. GPU or distributed features.

That gap is intentional. The project is being built in the order most likely to produce a durable system rather than a fragile showcase.

## Build Order

The intended implementation order is:

1. Codec.
2. Storage.
3. NVMe cache.
4. Catalog.
5. Index correctness.
6. Engine write and read paths.
7. Query layers, then server.

This order is part of the design, not a temporary preference.

## Getting Started

Current commands:

```bash
cargo run -- serve
cargo run -- info
cargo test
```

The current binary reports project information and reserves the server entrypoint for a later milestone. It is not yet a functional database server.

## Near-Term Goal

The next legitimate milestone is not “more features.” It is defining the durable segment format and its metadata invariants.

Until segment encoding and storage invariants are trustworthy, everything else is downstream complexity.

## Contributing

If you contribute to AetherDB, read these first:

1. [AGENTS.md](/Users/balasubramanyam/open-source/aetherdb/AGENTS.md)
2. [Architecture.md](/Users/balasubramanyam/open-source/aetherdb/Architecture.md)

If code and docs disagree, stop and reconcile them before adding more code.
