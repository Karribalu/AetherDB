#![allow(dead_code)]

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::{AetherError, Result};

/// Canonical namespace definition used by the catalog and segment codec.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamespaceMetadata {
    /// Logical namespace name exposed to clients.
    pub name: String,
    /// Active schema for the namespace.
    pub schema: SchemaMetadata,
    /// Time at which the namespace metadata was created.
    pub created_at: DateTime<Utc>,
}

impl NamespaceMetadata {
    pub fn validate(&self) -> Result<()> {
        validate_identifier("namespace", &self.name)?;
        self.schema.validate()
    }
}

/// Stable schema descriptor shared by catalog metadata and segment headers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaMetadata {
    /// Stable schema identifier independent from the current fingerprint.
    pub schema_id: Uuid,
    /// Monotonic schema version within the namespace.
    pub version: u32,
    /// Fingerprint of the encoded schema payload.
    pub fingerprint: SchemaFingerprint,
    /// Ordered field descriptors. Field IDs are stable across segment writes.
    pub fields: Vec<FieldMetadata>,
    /// Time at which this schema version was created.
    pub created_at: DateTime<Utc>,
}

impl SchemaMetadata {
    pub fn validate(&self) -> Result<()> {
        if self.version == 0 {
            return Err(codec_error("schema version must be greater than zero"));
        }

        if self.fields.is_empty() {
            return Err(codec_error("schema must define at least one field"));
        }

        self.fingerprint.validate()?;

        let mut field_ids = BTreeSet::new();
        let mut field_names = BTreeSet::new();

        for field in &self.fields {
            field.validate()?;

            if !field_ids.insert(field.field_id) {
                return Err(codec_error(format!(
                    "schema contains duplicate field id {}",
                    field.field_id
                )));
            }

            if !field_names.insert(field.name.as_str()) {
                return Err(codec_error(format!(
                    "schema contains duplicate field name '{}'",
                    field.name
                )));
            }
        }

        Ok(())
    }
}

/// Opaque schema fingerprint string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaFingerprint(pub String);

impl SchemaFingerprint {
    pub fn validate(&self) -> Result<()> {
        let fingerprint = self.0.trim();
        if fingerprint.is_empty() {
            return Err(codec_error("schema fingerprint must not be empty"));
        }

        if fingerprint.contains(char::is_whitespace) {
            return Err(codec_error(
                "schema fingerprint must not contain whitespace",
            ));
        }

        Ok(())
    }
}

/// Logical field descriptor stored as part of schema metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldMetadata {
    /// Stable field ID used by segment-level metadata.
    pub field_id: u32,
    /// User-facing field name.
    pub name: String,
    /// Logical data type.
    pub field_type: FieldType,
    /// Whether the field may be omitted or null in logical rows.
    pub nullable: bool,
    /// Whether raw values are materialized in columnar storage.
    pub stored: bool,
    /// Index structures requested for the field.
    pub index_kinds: Vec<FieldIndexKind>,
}

impl FieldMetadata {
    pub fn validate(&self) -> Result<()> {
        validate_identifier("field", &self.name)?;
        self.field_type.validate()?;

        if !self.stored && self.index_kinds.is_empty() {
            return Err(codec_error(format!(
                "field '{}' must be stored or indexed",
                self.name
            )));
        }

        let mut seen_index_kinds = BTreeSet::new();
        for index_kind in &self.index_kinds {
            if !seen_index_kinds.insert(index_kind) {
                return Err(codec_error(format!(
                    "field '{}' declares duplicate index kind {:?}",
                    self.name, index_kind
                )));
            }

            if !self.field_type.supports_index(index_kind) {
                return Err(codec_error(format!(
                    "field '{}' with type {:?} does not support index kind {:?}",
                    self.name, self.field_type, index_kind
                )));
            }
        }

        Ok(())
    }
}

/// Supported logical field types in the initial segment object model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FieldType {
    Text,
    Keyword,
    Boolean,
    Int64,
    UInt64,
    Float64,
    TimestampMicros,
    Bytes,
    Json,
    Vector {
        dimension: u32,
        distance_metric: VectorDistanceMetric,
    },
}

impl FieldType {
    fn validate(&self) -> Result<()> {
        if let Self::Vector { dimension, .. } = self {
            if *dimension == 0 {
                return Err(codec_error("vector field dimension must be greater than zero"));
            }
        }

        Ok(())
    }

    fn supports_index(&self, index_kind: &FieldIndexKind) -> bool {
        match index_kind {
            FieldIndexKind::FullText => matches!(self, Self::Text),
            FieldIndexKind::Numeric => {
                matches!(self, Self::Int64 | Self::UInt64 | Self::Float64 | Self::TimestampMicros)
            }
            FieldIndexKind::Bitmap => matches!(self, Self::Keyword | Self::Boolean),
            FieldIndexKind::Vector => matches!(self, Self::Vector { .. }),
        }
    }
}

/// Field-level index declarations supported by the first schema model.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldIndexKind {
    FullText,
    Numeric,
    Bitmap,
    Vector,
}

/// Distance metrics for vector fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VectorDistanceMetric {
    Cosine,
    DotProduct,
    Euclidean,
}

/// Immutable segment descriptor persisted in the catalog and segment footer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegmentMetadata {
    /// Unique segment identifier.
    pub segment_id: Uuid,
    /// Namespace to which the segment belongs.
    pub namespace: String,
    /// Schema identifier referenced by this segment.
    pub schema_id: Uuid,
    /// Fingerprint of the exact schema encoding used for the segment.
    pub schema_fingerprint: SchemaFingerprint,
    /// Object storage key holding the authoritative segment bytes.
    pub object_store_key: String,
    /// Segment creation time.
    pub created_at: DateTime<Utc>,
    /// Number of logical rows in the segment.
    pub row_count: u64,
    /// Total segment size in bytes.
    pub size_bytes: u64,
    /// Per-field storage and statistics metadata.
    pub fields: Vec<SegmentFieldMetadata>,
}

impl SegmentMetadata {
    pub fn validate(&self) -> Result<()> {
        validate_identifier("namespace", &self.namespace)?;
        self.schema_fingerprint.validate()?;

        if self.object_store_key.trim().is_empty() {
            return Err(codec_error("segment object storage key must not be empty"));
        }

        if self.row_count == 0 {
            return Err(codec_error("segment row count must be greater than zero"));
        }

        if self.size_bytes == 0 {
            return Err(codec_error("segment size must be greater than zero"));
        }

        let mut field_ids = BTreeSet::new();
        let mut ordinals = BTreeSet::new();

        for field in &self.fields {
            field.validate(self.row_count)?;

            if !field_ids.insert(field.field_id) {
                return Err(codec_error(format!(
                    "segment contains duplicate field id {}",
                    field.field_id
                )));
            }

            if !ordinals.insert(field.column_ordinal) {
                return Err(codec_error(format!(
                    "segment contains duplicate column ordinal {}",
                    field.column_ordinal
                )));
            }
        }

        Ok(())
    }
}

/// Field-level segment statistics stored alongside immutable segment metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SegmentFieldMetadata {
    /// Stable field ID from the schema.
    pub field_id: u32,
    /// Column position inside the segment.
    pub column_ordinal: u32,
    /// Encoded byte size of the field's column and index metadata.
    pub bytes_on_disk: u64,
    /// Count of non-null logical values in the segment.
    pub non_null_value_count: u64,
    /// Optional field statistics used for pruning and validation.
    pub statistics: Option<FieldStatistics>,
}

impl SegmentFieldMetadata {
    fn validate(&self, row_count: u64) -> Result<()> {
        if self.bytes_on_disk == 0 {
            return Err(codec_error(format!(
                "segment field {} must occupy at least one byte on disk",
                self.field_id
            )));
        }

        if self.non_null_value_count > row_count {
            return Err(codec_error(format!(
                "segment field {} has non-null count {} above row count {}",
                self.field_id, self.non_null_value_count, row_count
            )));
        }

        if let Some(statistics) = &self.statistics {
            statistics.validate(self.field_id, self.non_null_value_count, row_count)?;
        }

        Ok(())
    }
}

/// Generic field statistics carried in segment metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldStatistics {
    /// Number of null logical values in the segment.
    pub null_count: u64,
    /// Optional distinct value count if materialized during segment build.
    pub distinct_count: Option<u64>,
    /// Minimum value for pruning-compatible field types.
    pub min_value: Option<FieldValue>,
    /// Maximum value for pruning-compatible field types.
    pub max_value: Option<FieldValue>,
}

impl FieldStatistics {
    fn validate(&self, field_id: u32, non_null_value_count: u64, row_count: u64) -> Result<()> {
        if self.null_count > row_count {
            return Err(codec_error(format!(
                "field {} null count {} exceeds segment row count {}",
                field_id, self.null_count, row_count
            )));
        }

        if self.null_count + non_null_value_count > row_count {
            return Err(codec_error(format!(
                "field {} null count {} plus non-null count {} exceeds segment row count {}",
                field_id, self.null_count, non_null_value_count, row_count
            )));
        }

        if let Some(distinct_count) = self.distinct_count {
            if distinct_count > non_null_value_count {
                return Err(codec_error(format!(
                    "field {} distinct count {} exceeds non-null count {}",
                    field_id, distinct_count, non_null_value_count
                )));
            }
        }

        match (&self.min_value, &self.max_value) {
            (Some(min_value), Some(max_value)) => {
                if !min_value.same_variant(max_value) {
                    return Err(codec_error(format!(
                        "field {} min and max value types do not match",
                        field_id
                    )));
                }

                if min_value.sort_key() > max_value.sort_key() {
                    return Err(codec_error(format!(
                        "field {} min value exceeds max value",
                        field_id
                    )));
                }
            }
            (None, None) => {}
            _ => {
                return Err(codec_error(format!(
                    "field {} must define both min and max values together",
                    field_id
                )));
            }
        }

        Ok(())
    }
}

/// Comparable scalar values used in segment statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FieldValue {
    Keyword { value: String },
    Boolean { value: bool },
    Int64 { value: i64 },
    UInt64 { value: u64 },
    Float64 { value: f64 },
    TimestampMicros { value: i64 },
}

impl FieldValue {
    fn same_variant(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    fn sort_key(&self) -> SortKey<'_> {
        match self {
            Self::Keyword { value } => SortKey::String(value.as_str()),
            Self::Boolean { value } => SortKey::Bool(*value),
            Self::Int64 { value } => SortKey::Signed(*value),
            Self::UInt64 { value } => SortKey::Unsigned(*value),
            Self::Float64 { value } => SortKey::Float(*value),
            Self::TimestampMicros { value } => SortKey::Signed(*value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SortKey<'a> {
    String(&'a str),
    Bool(bool),
    Signed(i64),
    Unsigned(u64),
    Float(f64),
}

impl PartialOrd for SortKey<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::String(left), Self::String(right)) => left.partial_cmp(right),
            (Self::Bool(left), Self::Bool(right)) => left.partial_cmp(right),
            (Self::Signed(left), Self::Signed(right)) => left.partial_cmp(right),
            (Self::Unsigned(left), Self::Unsigned(right)) => left.partial_cmp(right),
            (Self::Float(left), Self::Float(right)) => left.partial_cmp(right),
            _ => None,
        }
    }
}

fn validate_identifier(kind: &str, value: &str) -> Result<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(codec_error(format!("{} identifier must not be empty", kind)));
    }

    if trimmed != value {
        return Err(codec_error(format!(
            "{} identifier must not have leading or trailing whitespace",
            kind
        )));
    }

    Ok(())
}

fn codec_error(message: impl Into<String>) -> AetherError {
    AetherError::Codec(message.into())
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn namespace_and_segment_metadata_validate_and_round_trip() {
        let namespace = sample_namespace();
        namespace.validate().expect("namespace metadata should validate");

        let encoded = serde_json::to_string(&namespace).expect("namespace metadata should encode");
        let decoded: NamespaceMetadata =
            serde_json::from_str(&encoded).expect("namespace metadata should decode");
        assert_eq!(decoded, namespace);

        let segment = sample_segment(&namespace.schema);
        segment.validate().expect("segment metadata should validate");

        let encoded = serde_json::to_string(&segment).expect("segment metadata should encode");
        let decoded: SegmentMetadata =
            serde_json::from_str(&encoded).expect("segment metadata should decode");
        assert_eq!(decoded, segment);
    }

    #[test]
    fn schema_rejects_duplicate_field_ids() {
        let mut schema = sample_namespace().schema;
        schema.fields[1].field_id = schema.fields[0].field_id;

        let error = schema.validate().expect_err("schema should reject duplicate field ids");
        assert!(error.to_string().contains("duplicate field id"));
    }

    #[test]
    fn field_rejects_incompatible_index_kind() {
        let field = FieldMetadata {
            field_id: 7,
            name: String::from("category"),
            field_type: FieldType::Keyword,
            nullable: false,
            stored: true,
            index_kinds: vec![FieldIndexKind::FullText],
        };

        let error = field.validate().expect_err("field should reject incompatible index kind");
        assert!(error.to_string().contains("does not support index kind"));
    }

    #[test]
    fn segment_rejects_invalid_statistics_bounds() {
        let namespace = sample_namespace();
        let mut segment = sample_segment(&namespace.schema);
        segment.fields[0].statistics = Some(FieldStatistics {
            null_count: 0,
            distinct_count: Some(3),
            min_value: Some(FieldValue::TimestampMicros { value: 50 }),
            max_value: Some(FieldValue::TimestampMicros { value: 10 }),
        });

        let error = segment
            .validate()
            .expect_err("segment should reject inverted min/max statistics");
        assert!(error.to_string().contains("min value exceeds max value"));
    }

    fn sample_namespace() -> NamespaceMetadata {
        NamespaceMetadata {
            name: String::from("articles"),
            schema: SchemaMetadata {
                schema_id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
                    .expect("static schema UUID should parse"),
                version: 1,
                fingerprint: SchemaFingerprint(String::from("schema:6e1f2c84d18d")),
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
                        name: String::from("embedding"),
                        field_type: FieldType::Vector {
                            dimension: 768,
                            distance_metric: VectorDistanceMetric::Cosine,
                        },
                        nullable: false,
                        stored: false,
                        index_kinds: vec![FieldIndexKind::Vector],
                    },
                ],
                created_at: Utc
                    .with_ymd_and_hms(2026, 3, 14, 12, 0, 0)
                    .single()
                    .expect("timestamp should be valid"),
            },
            created_at: Utc
                .with_ymd_and_hms(2026, 3, 14, 12, 0, 0)
                .single()
                .expect("timestamp should be valid"),
        }
    }

    fn sample_segment(schema: &SchemaMetadata) -> SegmentMetadata {
        SegmentMetadata {
            segment_id: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb")
                .expect("static segment UUID should parse"),
            namespace: String::from("articles"),
            schema_id: schema.schema_id,
            schema_fingerprint: schema.fingerprint.clone(),
            object_store_key: String::from("namespaces/articles/segments/segment-000001.aeseg"),
            created_at: Utc
                .with_ymd_and_hms(2026, 3, 14, 12, 5, 0)
                .single()
                .expect("timestamp should be valid"),
            row_count: 3,
            size_bytes: 4096,
            fields: vec![
                SegmentFieldMetadata {
                    field_id: 1,
                    column_ordinal: 0,
                    bytes_on_disk: 256,
                    non_null_value_count: 3,
                    statistics: None,
                },
                SegmentFieldMetadata {
                    field_id: 2,
                    column_ordinal: 1,
                    bytes_on_disk: 128,
                    non_null_value_count: 3,
                    statistics: Some(FieldStatistics {
                        null_count: 0,
                        distinct_count: Some(3),
                        min_value: Some(FieldValue::TimestampMicros { value: 10 }),
                        max_value: Some(FieldValue::TimestampMicros { value: 50 }),
                    }),
                },
                SegmentFieldMetadata {
                    field_id: 3,
                    column_ordinal: 2,
                    bytes_on_disk: 2048,
                    non_null_value_count: 3,
                    statistics: None,
                },
            ],
        }
    }
}