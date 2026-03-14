use aetherdb::codec::{
    FieldIndexKind, FieldMetadata, FieldStatistics, FieldType, FieldValue, NamespaceMetadata,
    SchemaFingerprint, SchemaMetadata, SegmentFieldMetadata, SegmentMetadata,
};
use chrono::TimeZone;
use uuid::Uuid;

pub fn sample_namespace() -> NamespaceMetadata {
    NamespaceMetadata {
        name: String::from("docs"),
        schema: SchemaMetadata {
            schema_id: Uuid::from_u128(0x11111111111111111111111111111111),
            version: 1,
            fingerprint: SchemaFingerprint(String::from("schema_docs_v1")),
            fields: vec![
                FieldMetadata {
                    field_id: 1,
                    name: String::from("title"),
                    field_type: FieldType::Text,
                    nullable: false,
                    stored: true,
                    index_kinds: vec![FieldIndexKind::FullText],
                },
                FieldMetadata {
                    field_id: 2,
                    name: String::from("published_at"),
                    field_type: FieldType::TimestampMicros,
                    nullable: false,
                    stored: true,
                    index_kinds: vec![FieldIndexKind::Numeric],
                },
                FieldMetadata {
                    field_id: 3,
                    name: String::from("category"),
                    field_type: FieldType::Keyword,
                    nullable: true,
                    stored: true,
                    index_kinds: vec![FieldIndexKind::Bitmap],
                },
            ],
            created_at: chrono::Utc
                .with_ymd_and_hms(2026, 3, 14, 12, 0, 0)
                .single()
                .expect("valid schema timestamp"),
        },
        created_at: chrono::Utc
            .with_ymd_and_hms(2026, 3, 14, 12, 0, 0)
            .single()
            .expect("valid namespace timestamp"),
    }
}

pub fn sample_segment() -> SegmentMetadata {
    let namespace = sample_namespace();

    SegmentMetadata {
        segment_id: Uuid::from_u128(0x22222222222222222222222222222222),
        namespace: namespace.name,
        schema_id: namespace.schema.schema_id,
        schema_fingerprint: namespace.schema.fingerprint,
        object_store_key: String::from("segments/docs/2026/03/14/segment-000001.seg"),
        created_at: chrono::Utc
            .with_ymd_and_hms(2026, 3, 14, 12, 5, 0)
            .single()
            .expect("valid segment timestamp"),
        row_count: 3,
        size_bytes: 4_096,
        fields: vec![
            SegmentFieldMetadata {
                field_id: 1,
                column_ordinal: 0,
                bytes_on_disk: 1_024,
                non_null_value_count: 3,
                statistics: None,
            },
            SegmentFieldMetadata {
                field_id: 2,
                column_ordinal: 1,
                bytes_on_disk: 512,
                non_null_value_count: 3,
                statistics: Some(FieldStatistics {
                    null_count: 0,
                    distinct_count: Some(3),
                    min_value: Some(FieldValue::TimestampMicros { value: 10 }),
                    max_value: Some(FieldValue::TimestampMicros { value: 30 }),
                }),
            },
            SegmentFieldMetadata {
                field_id: 3,
                column_ordinal: 2,
                bytes_on_disk: 256,
                non_null_value_count: 2,
                statistics: Some(FieldStatistics {
                    null_count: 1,
                    distinct_count: Some(2),
                    min_value: Some(FieldValue::Keyword {
                        value: String::from("guide"),
                    }),
                    max_value: Some(FieldValue::Keyword {
                        value: String::from("reference"),
                    }),
                }),
            },
        ],
    }
}