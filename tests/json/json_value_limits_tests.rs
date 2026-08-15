// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::MeasuredBudgetError;
use qubit_budget::ResourceLimit;
use qubit_budget::StructureLimits;
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;

/// Verifies that JSON value limits use machine-sized quantities by default.
#[test]
fn test_default_uses_usize_quantity() {
    let _: JsonValueLimits = JsonValueLimits::default();
}

/// Verifies that JSON convenience builders preserve a non-`usize` quantity.
#[test]
fn test_standard_builder_supports_u64_quantity() {
    let limits = JsonValueLimits::<JsonResource, u64>::unconfigured()
        .with_max_depth(1_u64)
        .with_max_nodes(2_u64)
        .with_max_sequence_items(3_u64)
        .with_max_map_entries(4_u64)
        .with_max_key_bytes(5_u64)
        .with_max_string_bytes(6_u64)
        .with_max_number_bytes(7_u64)
        .with_max_payload_bytes(8_u64);

    assert_eq!(limits.max_depth(), Some(1_u64));
    assert_eq!(limits.max_payload_bytes(), Some(8_u64));
    let _ = limits.budget();
}

/// Verifies that the standard JSON builder binds every value dimension.
#[test]
fn test_standard_builder_configures_all_value_dimensions() {
    let limits = JsonValueLimits::empty()
        .with_max_depth(1)
        .with_max_nodes(2)
        .with_max_sequence_items(3)
        .with_max_map_entries(4)
        .with_max_key_bytes(5)
        .with_max_string_bytes(6)
        .with_max_number_bytes(7)
        .with_max_payload_bytes(8);

    assert_eq!(limits.max_depth(), Some(1));
    assert_eq!(limits.max_nodes(), Some(2));
    assert_eq!(limits.max_sequence_items(), Some(3));
    assert_eq!(limits.max_map_entries(), Some(4));
    assert_eq!(limits.max_key_bytes(), Some(5));
    assert_eq!(limits.max_string_bytes(), Some(6));
    assert_eq!(limits.max_number_bytes(), Some(7));
    assert_eq!(limits.max_payload_bytes(), Some(8));
    assert_eq!(
        limits
            .string_bytes_limit()
            .expect("string limit")
            .resource(),
        &JsonResource::StringBytes,
    );
}

/// Verifies that the standard value builder creates an independent budget.
#[test]
fn test_standard_builder_creates_budget() {
    let mut budget = JsonValueLimits::empty().with_max_nodes(1).budget();
    let mut transaction = budget.transaction();

    transaction
        .try_admit(JsonMeasurement::Null { depth: 0 })
        .expect("one node fits");
    transaction.commit();
    assert_eq!(budget.used_nodes(), Some(1));
}

/// Verifies that structural limits may be borrowed or explicitly consumed.
#[test]
fn test_structure_limits_expresses_borrowing_and_ownership() {
    let limits = JsonValueLimits::empty().with_max_depth(4);
    let _: &StructureLimits<JsonResource, usize> = limits.structure_limits();
    assert_eq!(limits.structure_limits().max_depth(), Some(4));
    assert_eq!(limits.into_structure_limits().max_depth(), Some(4));
}

#[test]
fn test_empty_value_limits_report_unconfigured_maxima() {
    let limits = JsonValueLimits::empty();
    assert_eq!(limits.max_depth(), None);
    assert_eq!(limits.max_string_bytes(), None);
    assert_eq!(limits.max_number_bytes(), None);
    assert_eq!(limits.max_payload_bytes(), None);
}

#[test]
fn test_custom_resources_remain_attached_to_value_limits() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Resource {
        String,
        Number,
        Payload,
    }

    let limits = JsonValueLimits::<Resource, u8>::unconfigured()
        .with_string_bytes_limit(ResourceLimit::new(Resource::String, 3))
        .with_number_bytes_limit(ResourceLimit::new(Resource::Number, 4))
        .with_payload_bytes_limit(ResourceLimit::new(Resource::Payload, 5));

    assert_eq!(
        limits
            .string_bytes_limit()
            .expect("string limit")
            .resource(),
        &Resource::String,
    );
    assert_eq!(
        limits
            .number_bytes_limit()
            .expect("number limit")
            .resource(),
        &Resource::Number,
    );
    assert_eq!(
        limits
            .payload_bytes_limit()
            .expect("payload limit")
            .resource(),
        &Resource::Payload,
    );
}

/// Verifies point checks reject an oversized array without creating a budget.
#[test]
fn test_check_array_measurement_rejects_items_without_mutable_budget() {
    let limits = JsonValueLimits::empty().with_max_sequence_items(1);
    let error = limits
        .check(JsonMeasurement::Array { depth: 1, items: 2 })
        .expect_err("two items must exceed the point limit");
    assert_eq!(error.resource(), &JsonResource::SequenceItems);
}

/// Verifies unconfigured dimensions do not convert native measurements.
#[test]
fn test_unconfigured_payload_skips_unrepresentable_conversion() {
    let limits = JsonValueLimits::<JsonResource, u8>::unconfigured();
    limits
        .check(JsonMeasurement::String {
            depth: usize::MAX,
            bytes: usize::MAX,
        })
        .expect("unconfigured dimensions must not convert native values");
}

/// Verifies every measurement variant checks its matching point dimension.
#[test]
fn test_check_rejects_each_json_measurement_variant_at_point_limit() {
    let limits = JsonValueLimits::empty()
        .with_max_depth(1)
        .with_max_sequence_items(1)
        .with_max_map_entries(1)
        .with_max_key_bytes(1)
        .with_max_string_bytes(1)
        .with_max_number_bytes(1);
    let measurements = [
        (JsonMeasurement::Null { depth: 2 }, JsonResource::Depth),
        (JsonMeasurement::Boolean { depth: 2 }, JsonResource::Depth),
        (
            JsonMeasurement::String { depth: 1, bytes: 2 },
            JsonResource::StringBytes,
        ),
        (
            JsonMeasurement::Number { depth: 1, bytes: 2 },
            JsonResource::NumberBytes,
        ),
        (
            JsonMeasurement::Array { depth: 1, items: 2 },
            JsonResource::SequenceItems,
        ),
        (
            JsonMeasurement::Object {
                depth: 1,
                entries: 2,
            },
            JsonResource::MapEntries,
        ),
        (JsonMeasurement::Key { bytes: 2 }, JsonResource::KeyBytes),
    ];

    for (measurement, resource) in measurements {
        let error = limits
            .check(measurement)
            .expect_err("each measured dimension exceeds its point limit");
        assert!(matches!(error, MeasuredBudgetError::Budget(_)));
        assert_eq!(error.resource(), &resource);
    }
}

/// Verifies conversion failures precede depth and variant point checks.
#[test]
fn test_check_prioritizes_conversion_before_depth_and_point_limits() {
    let limits =
        JsonValueLimits::<JsonResource, u8>::unconfigured()
            .with_structure_limits(StructureLimits::empty().with_depth_limit(
                ResourceLimit::new(JsonResource::Depth, u8::MAX),
            ))
            .with_string_bytes_limit(ResourceLimit::new(
                JsonResource::StringBytes,
                u8::MAX,
            ));

    let error = limits
        .check(JsonMeasurement::String {
            depth: usize::from(u8::MAX) + 1,
            bytes: usize::from(u8::MAX) + 1,
        })
        .expect_err("depth conversion must reject before point checks");

    assert!(matches!(
        error,
        MeasuredBudgetError::Quantity {
            resource: JsonResource::Depth,
            ..
        }
    ));
}

/// Verifies depth point checks precede variant-specific point checks.
#[test]
fn test_check_prioritizes_depth_before_variant_point_limit() {
    let limits = JsonValueLimits::empty()
        .with_max_depth(1)
        .with_max_string_bytes(1);

    let error = limits
        .check(JsonMeasurement::String { depth: 2, bytes: 2 })
        .expect_err("depth must reject before string bytes");

    assert_eq!(error.resource(), &JsonResource::Depth);
}

/// Verifies a payload-only conversion error identifies the cumulative limit.
#[test]
fn test_check_payload_only_conversion_reports_payload_resource() {
    let limits = JsonValueLimits::<JsonResource, u8>::unconfigured()
        .with_payload_bytes_limit(ResourceLimit::new(
            JsonResource::PayloadBytes,
            u8::MAX,
        ));

    let error = limits
        .check(JsonMeasurement::String {
            depth: 0,
            bytes: usize::from(u8::MAX) + 1,
        })
        .expect_err(
            "configured payload conversion must reject oversized bytes",
        );

    assert!(matches!(
        error,
        MeasuredBudgetError::Quantity {
            resource: JsonResource::PayloadBytes,
            ..
        }
    ));
}

/// Verifies point limits select their resource before cumulative payload
/// limits.
#[test]
fn test_check_prefers_point_resource_when_payload_limit_is_also_configured() {
    let limits = JsonValueLimits::<JsonResource, u8>::unconfigured()
        .with_string_bytes_limit(ResourceLimit::new(
            JsonResource::StringBytes,
            u8::MAX,
        ))
        .with_payload_bytes_limit(ResourceLimit::new(
            JsonResource::PayloadBytes,
            u8::MAX,
        ));

    let error = limits
        .check(JsonMeasurement::String {
            depth: 0,
            bytes: usize::from(u8::MAX) + 1,
        })
        .expect_err("point resource must identify the conversion failure");

    assert!(matches!(
        error,
        MeasuredBudgetError::Quantity {
            resource: JsonResource::StringBytes,
            ..
        }
    ));
}

/// Verifies every configured point dimension reports its conversion resource.
#[test]
fn test_check_conversion_failures_report_each_point_resource() {
    let limits = JsonValueLimits::<JsonResource, u8>::unconfigured()
        .with_structure_limits(
            StructureLimits::empty()
                .with_sequence_items_limit(ResourceLimit::new(
                    JsonResource::SequenceItems,
                    u8::MAX,
                ))
                .with_map_entries_limit(ResourceLimit::new(
                    JsonResource::MapEntries,
                    u8::MAX,
                ))
                .with_key_bytes_limit(ResourceLimit::new(
                    JsonResource::KeyBytes,
                    u8::MAX,
                )),
        )
        .with_number_bytes_limit(ResourceLimit::new(
            JsonResource::NumberBytes,
            u8::MAX,
        ));
    let overflow = usize::from(u8::MAX) + 1;
    let measurements = [
        (
            JsonMeasurement::Number {
                depth: 0,
                bytes: overflow,
            },
            JsonResource::NumberBytes,
        ),
        (
            JsonMeasurement::Array {
                depth: 0,
                items: overflow,
            },
            JsonResource::SequenceItems,
        ),
        (
            JsonMeasurement::Object {
                depth: 0,
                entries: overflow,
            },
            JsonResource::MapEntries,
        ),
        (
            JsonMeasurement::Key { bytes: overflow },
            JsonResource::KeyBytes,
        ),
    ];

    for (measurement, resource) in measurements {
        let error = limits
            .check(measurement)
            .expect_err("configured native measurement must fit u8");
        assert!(matches!(
            error,
            MeasuredBudgetError::Quantity {
                resource: actual,
                ..
            } if actual == resource
        ));
    }
}
