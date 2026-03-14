use aetherdb::codec::{
    FieldIndexKind, FieldMetadata, FieldStatistics, FieldType, FieldValue, SchemaFingerprint,
    SegmentFieldMetadata, SegmentMetadata,
};
use chrono::TimeZone;
use criterion::{Criterion, criterion_group, criterion_main};
use uuid::Uuid;

fn sample_segment() -> SegmentMetadata {
    SegmentMetadata {
        segment_id: Uuid::from_u128(0x33333333333333333333333333333333),
        namespace: String::from("bench_docs"),
        schema_id: Uuid::from_u128(0x44444444444444444444444444444444),
        schema_fingerprint: SchemaFingerprint(String::from("schema_bench_v1")),
        object_store_key: String::from("segments/bench_docs/segment-000001.seg"),
        created_at: chrono::Utc
            .with_ymd_and_hms(2026, 3, 14, 12, 15, 0)
            .single()
            .expect("valid benchmark timestamp"),
        row_count: 10_000,
        size_bytes: 2 * 1024 * 1024,
        fields: vec![
            SegmentFieldMetadata {
                field_id: 1,
                column_ordinal: 0,
                bytes_on_disk: 640_000,
                non_null_value_count: 10_000,
                statistics: None,
            },
            SegmentFieldMetadata {
                field_id: 2,
                column_ordinal: 1,
                bytes_on_disk: 128_000,
                non_null_value_count: 10_000,
                statistics: Some(FieldStatistics {
                    null_count: 0,
                    distinct_count: Some(8_000),
                    min_value: Some(FieldValue::Int64 { value: 1 }),
                    max_value: Some(FieldValue::Int64 { value: 10_000 }),
                }),
            },
            SegmentFieldMetadata {
                field_id: 3,
                column_ordinal: 2,
                bytes_on_disk: 32_000,
                non_null_value_count: 9_000,
                statistics: Some(FieldStatistics {
                    null_count: 1_000,
                    distinct_count: Some(12),
                    min_value: Some(FieldValue::Keyword {
                        value: String::from("alpha"),
                    }),
                    max_value: Some(FieldValue::Keyword {
                        value: String::from("omega"),
                    }),
                }),
            },
        ],
    }
}

fn benchmark_segment_validate(c: &mut Criterion) {
    let segment = sample_segment();

    c.bench_function("segment_metadata_validate", |b| {
        b.iter(|| segment.validate().expect("benchmark segment should validate"))
    });
}

fn benchmark_field_metadata_validate(c: &mut Criterion) {
    let field = FieldMetadata {
        field_id: 99,
        name: String::from("body"),
        field_type: FieldType::Text,
        nullable: true,
        stored: true,
        index_kinds: vec![FieldIndexKind::FullText],
    };

    c.bench_function("field_metadata_validate", |b| {
        b.iter(|| field.validate().expect("benchmark field should validate"))
    });
}

criterion_group!(metadata_validation, benchmark_segment_validate, benchmark_field_metadata_validate);
criterion_main!(metadata_validation);