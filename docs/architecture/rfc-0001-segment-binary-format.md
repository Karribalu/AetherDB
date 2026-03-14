# RFC 0001: Segment Binary Format v1alpha1

## Status

Proposed

## Last Updated

2026-03-14

## Summary

This RFC defines the first implementation target for the AetherDB segment binary format.

The format is designed for immutable segment objects whose authoritative copy lives in object storage and whose local NVMe copies are disposable cache entries. The format must therefore be:

1. Self-describing enough to reject incompatible or partial reads deterministically.
2. Stable enough to support early storage and catalog work.
3. Structured enough to hold column data plus multiple index families in one segment object.
4. Strict about invariants before any optimization work begins.

This RFC is the Week 1 format contract. It is not the final byte-level primitive specification for every region. Week 2 and Week 3 work will define the exact primitive encodings, region payload schemas, and corruption tests that implement this contract.

## Motivation

Everything downstream depends on the segment format:

1. `storage` cannot safely read or write segment objects until the file boundary, checksum policy, and discovery rules are explicit.
2. `catalog` cannot reference segment objects durably until segment identity and schema binding are part of the format contract.
3. `index` implementations need stable region boundaries so index blobs can be written and read without inventing ad hoc layouts later.
4. `engine` read and write paths depend on deterministic segment opening, validation, and field lookup.

If the segment format is vague, every later subsystem will encode hidden assumptions that become migration pain.

## Goals

1. Define a single immutable segment object layout for object storage and cache copies.
2. Make format compatibility checks explicit and cheap.
3. Separate header metadata, body regions, and footer directory responsibilities.
4. Bind the segment bytes to the existing metadata model in `src/codec/metadata.rs`.
5. Make partial writes and corruption detectable before a segment is trusted.
6. Leave room for forward-compatible region additions without changing the entire format.

## Non-Goals

1. Finalize the binary encoding of varints, delta streams, or every primitive container.
2. Finalize compression codecs.
3. Define merge-time tombstone handling.
4. Define cross-segment manifests or catalog update semantics.
5. Define the internal payload layout of HNSW, BKD, bitmap, or inverted index regions beyond their placement in the segment.

## Format Label

The first segment format is named `v1alpha1`.

The on-disk version identifier is a compact numeric value with this mapping:

- `1` => `v1alpha1`

Human-facing documentation uses `v1alpha1`. The binary file stores the numeric identifier.

## Design Principles

1. Object storage is the source of truth. Segment bytes must be portable across local cache and remote storage without reinterpretation.
2. Segments are immutable. Any field, index, or metadata change produces a new segment object.
3. Readers must fail closed. If required checks fail, the segment is unreadable.
4. The format must permit selective reads. Readers should be able to inspect metadata and locate regions without scanning the full file.
5. Region boundaries must be explicit so new index families do not force a redesign of the outer file format.

## Segment Layout

Each segment file has four logical parts:

```text
┌──────────────────────────────────────────────────────────────┐
│ Preamble       fixed-size format identifier and sizing      │
│ Header         variable-size metadata needed to open        │
│ Body           variable-size column and index regions       │
│ Footer         variable-size region directory and checks    │
│ Footer CRC32   fixed-size checksum for the footer payload   │
└──────────────────────────────────────────────────────────────┘
```

Concrete read order:

1. Read the preamble.
2. Validate magic bytes and format version.
3. Read and decode the header using the length recorded in the preamble.
4. Use the footer offset recorded in the header to seek directly to the footer.
5. Validate the footer checksum.
6. Read the region directory from the footer.
7. Read only the body regions required by the caller.

This allows fast segment opening without scanning the entire file.

## Preamble

The preamble is fixed-size and appears at byte offset `0`.

Required fields:

| Field            | Size    | Purpose                                                   |
| ---------------- | ------- | --------------------------------------------------------- |
| `magic`          | 8 bytes | ASCII `AETHERDB`                                          |
| `format_version` | 1 byte  | Numeric segment format version                            |
| `header_len`     | 4 bytes | Length in bytes of the encoded header                     |
| `flags`          | 4 bytes | Format flags; all zero in `v1alpha1`                      |
| `reserved`       | 8 bytes | Reserved for future compatibility; all zero in `v1alpha1` |

The preamble is intentionally simple. It exists to answer three questions immediately:

1. Is this an AetherDB segment?
2. Can this reader understand the format version?
3. How many bytes must be read to decode the header?

If any reserved bits are non-zero in `v1alpha1`, the reader must reject the segment as unsupported.

## Header

The header is the first variable-sized structure in the file. It contains the metadata required to open the segment safely and bind it to the correct schema.

The header is authoritative for:

1. Segment identity.
2. Namespace identity.
3. Exact schema identity.
4. Ordered field descriptors.
5. Global counts needed before body reads.
6. The absolute offset of the footer.

### Header Payload

The header must encode the following logical fields:

| Logical field        | Source model                                                          |
| -------------------- | --------------------------------------------------------------------- |
| `segment_id`         | `SegmentMetadata.segment_id`                                          |
| `namespace`          | `SegmentMetadata.namespace`                                           |
| `schema_id`          | `SegmentMetadata.schema_id`                                           |
| `schema_version`     | `SchemaMetadata.version`                                              |
| `schema_fingerprint` | `SchemaMetadata.fingerprint` and `SegmentMetadata.schema_fingerprint` |
| `created_at`         | `SegmentMetadata.created_at`                                          |
| `row_count`          | `SegmentMetadata.row_count`                                           |
| `field_count`        | derived from `SchemaMetadata.fields.len()`                            |
| `footer_offset`      | segment-local absolute byte offset                                    |
| `field_descriptors`  | `SchemaMetadata.fields`                                               |

### Header Rules

1. The header must include the full ordered `SchemaMetadata.fields` list so readers can map field IDs, names, types, and declared index kinds without consulting the catalog.
2. The header must not include mutable storage-state concepts such as cache location or query-planning hints.
3. The header must be sufficient to fail early if the namespace or schema fingerprint does not match what the catalog expects.
4. `footer_offset` must point to the first byte of the footer payload, not the checksum trailer.

### Why Schema Lives In The Header

Schema binding belongs in the header because:

1. Field resolution is needed before any body region can be interpreted.
2. Storage and cache layers need deterministic open-time validation.
3. A segment must remain self-describing even if a later schema version exists in the catalog.

## Body

The body stores the actual columnar data and index payloads. The outer segment format treats the body as an ordered sequence of opaque regions. The footer directory explains where each region begins, how long it is, and what it represents.

In `v1alpha1`, body regions are append-only within a single segment file and are never modified in place.

### Region Categories

The first format reserves region kinds for the following categories:

| Region kind          | Description                                                    |
| -------------------- | -------------------------------------------------------------- |
| `column_values`      | Stored field values for one field                              |
| `column_nulls`       | Null bitmap or equivalent null representation for one field    |
| `column_offsets`     | Offset table for variable-width stored values                  |
| `inverted_terms`     | Text dictionary for one field                                  |
| `inverted_postings`  | Postings and term statistics for one field                     |
| `bkd_tree`           | Numeric or timestamp tree payload for one field                |
| `bitmap_dictionary`  | Encoded value dictionary for one field                         |
| `bitmap_postings`    | Bitmap containers for one field                                |
| `vector_values`      | Raw vector column payload for one field                        |
| `hnsw_graph`         | HNSW graph payload for one field                               |
| `metadata_extension` | Future metadata payloads not required for open-time validation |

Not every segment will contain every region kind.

### Body Ordering Rule

The initial ordering rule is:

1. Stored column regions first.
2. Index regions second.
3. Optional metadata extension regions last.

Within each category, regions are ordered by ascending `field_id` and then by region kind.

This ordering is not primarily a performance optimization. It is a determinism rule so the same logical input produces byte-reproducible region ordering.

## Footer

The footer is the authoritative directory for the body.

The footer must contain:

1. The serialized `SegmentMetadata` value for the segment.
2. A region directory entry for every body region.
3. Per-region integrity metadata.
4. Footer-local counts needed to validate completeness.

### Footer Payload

The footer payload includes two logical parts:

1. `segment_metadata`: the durable `SegmentMetadata` structure, including `SegmentFieldMetadata` and field statistics.
2. `region_directory`: a list of region descriptors.

### Region Directory Entry

Each region directory entry must encode at least:

| Field         | Purpose                                                   |
| ------------- | --------------------------------------------------------- |
| `region_kind` | Identifies how the payload should be interpreted          |
| `field_id`    | Field to which the region belongs, if applicable          |
| `offset`      | Absolute segment byte offset                              |
| `length`      | Payload length in bytes                                   |
| `checksum`    | CRC32 for the region payload                              |
| `flags`       | Region-local flags; all zero in `v1alpha1` unless defined |

### Footer Rules

1. Every body region must have exactly one footer directory entry.
2. No directory entry may point outside the segment file bounds.
3. Region byte ranges must not overlap.
4. Directory entries must be sorted by ascending `offset`.
5. `SegmentMetadata.size_bytes` must equal the actual file size.
6. `SegmentFieldMetadata.bytes_on_disk` must reconcile with the sum of directory entries attributed to that field.

### Why Metadata Lives In The Footer

`SegmentMetadata` belongs in the footer because it depends on finalized byte sizes and field statistics that are not fully known until the segment assembly pass is complete.

## Footer Checksum

The final 4 bytes of the file are a CRC32 of the footer payload only.

This does not replace per-region checksums. It serves a narrower purpose:

1. Detect truncated or corrupted footer reads quickly.
2. Protect the directory and finalized `SegmentMetadata`, which are required before trusting any body offsets.

The whole-file checksum policy is explicitly deferred. `v1alpha1` requires footer CRC32 plus per-region CRC32.

## Metadata Mapping

This RFC intentionally reuses the existing codec metadata model instead of defining a separate document-only model.

### Namespace Metadata

`NamespaceMetadata` is not stored as a standalone top-level payload in the first segment file format. The segment stores the namespace name in the header and relies on the catalog to own the namespace record itself.

Reasoning:

1. The segment must bind to one namespace, but it does not need to embed the entire namespace document for safe reads.
2. The catalog remains the authority for namespace lifecycle.

### Schema Metadata

The schema record is split across existing types as follows:

1. `schema_id`, `schema_version`, and `schema_fingerprint` are recorded in the header.
2. The ordered `FieldMetadata` list is recorded in the header.
3. The segment footer echoes the exact `schema_fingerprint` through `SegmentMetadata` validation.

### Segment Metadata

`SegmentMetadata` is the durable summary of the finalized object and lives in the footer because it includes `size_bytes` and `SegmentFieldMetadata.bytes_on_disk`, which depend on the completed body layout.

## Integrity And Failure Rules

The format must fail closed under these conditions:

1. Magic bytes mismatch.
2. Unknown format version.
3. Non-zero reserved fields in `v1alpha1`.
4. Header decode failure.
5. Footer offset outside file bounds.
6. Footer CRC32 mismatch.
7. Missing region directory entry for a referenced field payload.
8. Overlapping or out-of-bounds region ranges.
9. Region CRC32 mismatch.
10. Schema fingerprint mismatch against the catalog's active schema for the referenced segment.

Operational consequence:

1. A failed cache copy is discarded and re-fetched from object storage.
2. A failed object storage segment is treated as unreadable and must block segment activation or query use until reconciled.

Partial segment writes are never valid segments.

## Determinism Rules

For a fixed logical input and fixed codec version, segment bytes must be reproducible.

The format-level determinism requirements are:

1. Field descriptors are written in stable schema order.
2. Body regions are written in deterministic category and `field_id` order.
3. Footer directory entries are written in ascending offset order.
4. Checksums are computed over exact payload bytes with no environment-dependent variation.

This RFC does not yet define how timestamps used in metadata are sourced. The write path must make that explicit before claiming full byte reproducibility in tests.

## Forward Compatibility Policy

`v1alpha1` uses a conservative compatibility policy.

Readers may accept a segment only if:

1. `format_version == 1`
2. All reserved preamble bytes are zero.
3. All unknown footer or region flags are zero.

Writers targeting `v1alpha1` must:

1. Write zero into all reserved fields.
2. Avoid emitting region kinds outside this RFC.
3. Refuse to label the file as `v1alpha1` if it depends on semantics not defined here.

This is intentionally strict. The first version prioritizes safety over permissive parsing.

## Write Protocol Expectations

This RFC only defines the file format, but two write-side expectations are part of the format contract:

1. The writer must construct the full segment object before publishing it to object storage.
2. The catalog must not advertise a segment whose bytes fail this RFC's validation rules.

This is how the binary format participates in the broader durability rule that object storage is the source of truth.

## Deferred Decisions

The following details are intentionally deferred to subsequent codec implementation work:

1. Exact primitive encodings for integers, lengths, strings, timestamps, and UUIDs.
2. Whether region CRC32 remains the permanent checksum choice or upgrades to CRC32C or xxHash-based validation later.
3. Compression framing for large column and postings regions.
4. Whether a mirrored trailer should be added for reverse scans in a later format version.
5. Whether footer payloads should be length-prefixed in addition to being located through `footer_offset`.

## Implementation Plan

### Week 2

1. Define the exact header and footer structs in `codec`.
2. Define primitive encoding rules for numeric fields, strings, UUIDs, and timestamps.
3. Define `RegionKind` and `RegionDirectoryEntry` enums and structs.
4. Add round-trip tests for primitive encoding and small header/footer fixtures.

### Week 3

1. Implement header and footer serialization.
2. Implement metadata serialization for `SchemaMetadata`, `FieldMetadata`, and `SegmentMetadata` payloads.
3. Add corruption tests for bad magic, bad version, bad footer offset, and bad footer checksum.
4. Add validation tests for overlapping and out-of-bounds regions.

### Week 4

1. Build a full fixture segment containing at least one stored field and one index region.
2. Validate complete open-path round trips.
3. Lock `v1alpha1` as the baseline format for storage work.

## Alternatives Considered

### Alternative 1: Put Offsets Only In The Header

Rejected for the first format.

Reason:

1. Final region sizes and field statistics are naturally known at the end of assembly.
2. A footer-centered directory makes segment finalization simpler and keeps finalized metadata together.

### Alternative 2: Store No Schema In The Segment

Rejected.

Reason:

1. Readers need self-contained field typing and index declarations.
2. Cache validation should not require a round trip to reconstruct field interpretation.

### Alternative 3: One Region Per Field With Mixed Payloads

Rejected for the first format.

Reason:

1. It would make partial reads and later index-specific evolution more awkward.
2. Separate region kinds provide cleaner validation and more explicit boundaries.

## Acceptance Criteria

This RFC is satisfied when all of the following are true:

1. A `codec` implementation can serialize and deserialize the preamble, header, footer, and region directory deterministically.
2. A segment open path can reject corruption and partial writes according to the failure rules above.
3. A segment can bind to the existing metadata model without undocumented translation layers.
4. The format is stable enough for `storage` to persist and retrieve opaque segment blobs.

## Consequences

Good consequences:

1. Storage work can begin with a concrete object contract.
2. Catalog work gets explicit segment identity and schema-binding expectations.
3. Index implementations inherit a stable outer container and only need to define their inner region encodings.

Costs:

1. The format is stricter than a prototype-oriented layout and will reject segments aggressively.
2. Some low-level choices remain open until Week 2 and Week 3.
3. Early implementations must honor determinism and checksum rules before chasing performance.
