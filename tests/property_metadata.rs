mod common;

use aetherdb::codec::{FieldStatistics, FieldValue};
use proptest::prelude::*;

proptest! {
    #[test]
    fn field_statistics_reject_null_counts_above_row_count(
        row_count in 1_u64..10_000,
        overflow in 1_u64..1_000,
        non_null_value_count in 0_u64..10_000,
    ) {
        let statistics = FieldStatistics {
            null_count: row_count.saturating_add(overflow),
            distinct_count: None,
            min_value: Some(FieldValue::Int64 { value: 1 }),
            max_value: Some(FieldValue::Int64 { value: 5 }),
        };

        let mut segment = common::sample_segment();
        segment.row_count = row_count;
        segment.fields[1].non_null_value_count = non_null_value_count.min(row_count);
        segment.fields[1].statistics = Some(statistics);

        prop_assert!(segment.validate().is_err());
    }

    #[test]
    fn valid_sample_segment_survives_json_round_trip(suffix in "[a-z0-9_]{1,12}") {
        let mut segment = common::sample_segment();
        segment.object_store_key = format!("segments/docs/{suffix}.seg");

        let encoded = serde_json::to_string(&segment).expect("segment should encode");
        let decoded: aetherdb::codec::SegmentMetadata =
            serde_json::from_str(&encoded).expect("segment should decode");

        prop_assert_eq!(&decoded, &segment);
        prop_assert!(decoded.validate().is_ok());
    }
}