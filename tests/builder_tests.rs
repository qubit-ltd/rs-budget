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
    assert_eq!(structure.max_key_bytes(), Some(5));

    let string = StringLimits::<StructureResource, u64>::builder()
        .utf8_bytes_limit(ResourceLimit::new(StructureResource::KeyBytes, 6_u64))
        .build();
    assert_eq!(string.utf8_bytes_limit().unwrap().maximum(), 6);
    let _: StringLimits<StructureResource, u64> = Default::default();
    let _: StringLimitsBuilder<StructureResource, u64> = Default::default();

    #[cfg(feature = "json")]
    {
        use qubit_budget::json::JsonDecodeLimits;
        use qubit_budget::json::JsonEncodeLimits;
        use qubit_budget::json::JsonResource;
        use qubit_budget::json::JsonValueLimits;

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
            .normalized_input_bytes_limit(ResourceLimit::new(
                JsonResource::NormalizedInputBytes,
                19,
            ))
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
        assert!(
            decimal
                .coefficient_limits()
                .magnitude_bits_limit()
                .is_some()
        );
        let _: BigDecimalLimits<StructureResource, u64> = Default::default();
        let _: BigDecimalLimitsBuilder<StructureResource, u64> = Default::default();
    }
}
