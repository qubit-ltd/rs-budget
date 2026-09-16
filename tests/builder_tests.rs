// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::ResourceLimit;
use qubit_budget::StringLimits;
use qubit_budget::StringLimitsBuilder;
use qubit_budget::StructureLimits;
use qubit_budget::StructureResource;

#[test]
fn builders_cover_generic_limit_setters() {
    let structure = StructureLimits::<StructureResource, usize>::builder()
        .depth_limit(ResourceLimit::new(StructureResource::Depth, 1))
        .nodes_limit(ResourceLimit::new(StructureResource::Nodes, 2))
        .sequence_items_limit(ResourceLimit::new(StructureResource::SequenceItems, 3))
        .map_entries_limit(ResourceLimit::new(StructureResource::MapEntries, 4))
        .key_bytes_limit(ResourceLimit::new(StructureResource::KeyBytes, 5))
        .build();
    assert_eq!(structure.max_depth(), Some(1));
    assert_eq!(structure.max_key_bytes(), Some(5));
    let structure = structure
        .into_builder()
        .max_depth(6)
        .max_nodes(7)
        .max_sequence_items(8)
        .max_map_entries(9)
        .max_key_bytes(10)
        .build();
    assert_eq!(structure.max_depth(), Some(6));
    assert_eq!(structure.max_key_bytes(), Some(10));
    assert_eq!(structure.max_nodes(), Some(7));

    let string = StringLimits::<StructureResource, u64>::builder()
        .utf8_bytes_limit(ResourceLimit::new(StructureResource::KeyBytes, 6_u64))
        .build();
    assert_eq!(string.utf8_bytes_limit().unwrap().maximum(), 6);
    assert_eq!(
        string
            .into_builder()
            .build()
            .utf8_bytes_limit()
            .map(ResourceLimit::maximum),
        Some(6)
    );
    let _: StringLimits<StructureResource, u64> = Default::default();
    let _: StringLimitsBuilder<StructureResource, u64> = Default::default();

    #[cfg(feature = "json")]
    {
        use qubit_budget::json::JsonDecodeLimits;
        use qubit_budget::json::JsonEncodeLimits;
        use qubit_budget::json::JsonMeasurement;
        use qubit_budget::json::JsonResource;
        use qubit_budget::json::JsonValueLimits;
        use qubit_budget::json::JsonValueLimitsBuilder;

        let value = JsonValueLimits::<JsonResource, usize>::builder()
            .string_bytes_limit(ResourceLimit::new(JsonResource::StringBytes, 7))
            .number_bytes_limit(ResourceLimit::new(JsonResource::NumberBytes, 8))
            .payload_bytes_limit(ResourceLimit::new(JsonResource::PayloadBytes, 9))
            .structure_limits(
                StructureLimits::<JsonResource, usize>::builder()
                    .nodes_limit(ResourceLimit::new(JsonResource::Nodes, 25))
                    .build(),
            )
            .max_depth(10)
            .max_nodes(11)
            .max_sequence_items(12)
            .max_map_entries(13)
            .max_key_bytes(14)
            .max_string_bytes(15)
            .max_number_bytes(16)
            .max_payload_bytes(17)
            .build();
        let decode = JsonDecodeLimits::<JsonResource, usize>::builder()
            .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, 18))
            .normalized_input_bytes_limit(ResourceLimit::new(JsonResource::NormalizedInputBytes, 19))
            .value_limits(value)
            .build();
        let encode = JsonEncodeLimits::<JsonResource, usize>::builder()
            .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 20))
            .value_limits(value)
            .build();
        assert_eq!(decode.max_input_bytes(), Some(18));
        assert_eq!(encode.max_output_bytes(), Some(20));
        assert!(decode.normalized_input_bytes_limit().is_some());
        assert!(decode.value_limits().max_nodes().is_some());
        assert!(encode.value_limits().max_nodes().is_some());
        let rebuilt_decode = decode
            .into_builder()
            .max_input_bytes(21_usize)
            .max_depth(22_usize)
            .build();
        assert_eq!(rebuilt_decode.max_input_bytes(), Some(21));
        assert_eq!(rebuilt_decode.value_limits().max_depth(), Some(22));
        let rebuilt_encode = encode
            .into_builder()
            .max_output_bytes(23_usize)
            .max_nodes(24_usize)
            .build();
        assert_eq!(rebuilt_encode.max_output_bytes(), Some(23));
        assert_eq!(rebuilt_encode.value_limits().max_nodes(), Some(24));

        let generic_decode = JsonDecodeLimits::<JsonResource, u64>::builder()
            .max_input_bytes(25)
            .max_normalized_input_bytes(26)
            .max_depth(27)
            .max_nodes(28)
            .max_sequence_items(29)
            .max_map_entries(30)
            .max_key_bytes(31)
            .max_string_bytes(32)
            .max_number_bytes(33)
            .max_payload_bytes(34)
            .build();
        assert_eq!(generic_decode.max_input_bytes(), Some(25));
        assert_eq!(generic_decode.value_limits().max_payload_bytes(), Some(34));

        let generic_encode = JsonEncodeLimits::<JsonResource, u64>::builder()
            .max_output_bytes(35)
            .max_depth(36)
            .max_nodes(37)
            .max_sequence_items(38)
            .max_map_entries(39)
            .max_key_bytes(40)
            .max_string_bytes(41)
            .max_number_bytes(42)
            .max_payload_bytes(43)
            .build();
        assert_eq!(generic_encode.max_output_bytes(), Some(35));
        assert_eq!(generic_encode.value_limits().max_number_bytes(), Some(42));

        let generic_value = JsonValueLimits::<JsonResource, u64>::builder()
            .structure_limits(
                StructureLimits::<JsonResource, u64>::builder()
                    .nodes_limit(ResourceLimit::new(JsonResource::Nodes, 44))
                    .build(),
            )
            .max_nodes(45)
            .build();
        assert_eq!(generic_value.max_nodes(), Some(45));
        let generic_value = JsonValueLimitsBuilder::<JsonResource, u64>::new()
            .structure_limits(
                StructureLimits::<JsonResource, u64>::builder()
                    .nodes_limit(ResourceLimit::new(JsonResource::Nodes, 48))
                    .build(),
            )
            .build();
        assert_eq!(generic_value.structure_limits().max_nodes(), Some(48));
        let _generic_budget = JsonValueLimits::<JsonResource, u64>::builder().max_nodes(46).budget();
        let mut generic_budget = JsonValueLimits::<JsonResource, u64>::builder().max_nodes(47).budget();
        let mut transaction = generic_budget.transaction();
        transaction
            .try_admit(JsonMeasurement::Null { depth: 1 })
            .expect("generic value admission should fit");
        assert_eq!(transaction.used_nodes(), Some(1));
        transaction.commit().expect("successful transaction commits");
        assert_eq!(generic_budget.used_nodes(), Some(1));

        assert_eq!(value.into_builder().build(), value);
        let _: JsonDecodeLimits<JsonResource, usize> = Default::default();
        let _: JsonEncodeLimits<JsonResource, usize> = Default::default();
        let _: JsonValueLimits<JsonResource, usize> = Default::default();
    }

    #[cfg(feature = "big-integer")]
    {
        use qubit_budget::BigIntegerLimits;
        use qubit_budget::BigIntegerLimitsBuilder;
        let integer = BigIntegerLimits::<StructureResource, u64>::builder()
            .magnitude_bits_limit(ResourceLimit::new(StructureResource::Nodes, 21_u64))
            .significant_decimal_digits_limit(ResourceLimit::new(StructureResource::Nodes, 22_u64))
            .build();
        assert_eq!(integer.magnitude_bits_limit().unwrap().maximum(), 21);
        assert!(integer.significant_decimal_digits_limit().is_some());
        assert_eq!(integer.into_builder().build(), integer);
        let _: BigIntegerLimits<StructureResource, u64> = Default::default();
        let _: BigIntegerLimitsBuilder<StructureResource, u64> = Default::default();
    }

    #[cfg(feature = "big-decimal")]
    {
        use qubit_budget::BigDecimalLimits;
        use qubit_budget::BigDecimalLimitsBuilder;
        use qubit_budget::BigIntegerLimits;
        let decimal = BigDecimalLimits::<StructureResource, u64>::builder()
            .coefficient_limits(
                BigIntegerLimits::builder()
                    .magnitude_bits_limit(ResourceLimit::new(StructureResource::Nodes, 23_u64))
                    .build(),
            )
            .scale_magnitude_limit(ResourceLimit::new(StructureResource::Nodes, 24_u64))
            .build();
        assert_eq!(decimal.scale_magnitude_limit().unwrap().maximum(), 24);
        assert!(decimal.coefficient_limits().magnitude_bits_limit().is_some());
        assert_eq!(decimal.into_builder().build(), decimal);
        let _: BigDecimalLimits<StructureResource, u64> = Default::default();
        let _: BigDecimalLimitsBuilder<StructureResource, u64> = Default::default();
    }
}
