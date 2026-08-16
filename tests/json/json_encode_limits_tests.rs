// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;

/// Verifies that JSON encode limits use machine-sized quantities by default.
#[test]
fn test_default_uses_usize_quantity() {
    let _: JsonEncodeLimits = JsonEncodeLimits::default();
}

/// Verifies that encode builders configure output and nested value limits.
#[test]
fn test_standard_builder_configures_encode_dimensions() {
    let limits = JsonEncodeLimits::<JsonResource, usize>::new()
        .with_max_output_bytes(1)
        .with_max_depth(2)
        .with_max_nodes(3)
        .with_max_sequence_items(4)
        .with_max_map_entries(5)
        .with_max_key_bytes(6)
        .with_max_string_bytes(7)
        .with_max_number_bytes(8)
        .with_max_payload_bytes(9);

    assert_eq!(limits.max_output_bytes(), Some(1));
    assert_eq!(limits.value_limits().max_depth(), Some(2));
    assert_eq!(limits.value_limits().max_nodes(), Some(3));
    assert_eq!(limits.value_limits().max_sequence_items(), Some(4));
    assert_eq!(limits.value_limits().max_map_entries(), Some(5));
    assert_eq!(limits.value_limits().max_key_bytes(), Some(6));
    assert_eq!(limits.value_limits().max_string_bytes(), Some(7));
    assert_eq!(limits.value_limits().max_number_bytes(), Some(8));
    assert_eq!(limits.value_limits().max_payload_bytes(), Some(9));
}

/// Verifies that nested value limits may be borrowed or explicitly consumed.
#[test]
fn test_value_limits_expresses_borrowing_and_ownership() {
    let limits =
        JsonEncodeLimits::<JsonResource, usize>::new().with_max_depth(2);
    let _: &JsonValueLimits = limits.value_limits();
    assert_eq!(limits.value_limits().max_depth(), Some(2));
    assert_eq!(limits.into_value_limits().max_depth(), Some(2));
}

#[test]
fn test_empty_encode_limits_report_unconfigured_maximum() {
    assert_eq!(
        JsonEncodeLimits::<JsonResource, usize>::new().max_output_bytes(),
        None
    );
}

#[test]
fn test_generic_encode_limits_expose_maximum() {
    let limits =
        JsonEncodeLimits::<JsonResource, u8>::new().with_output_bytes_limit(
            ResourceLimit::new(JsonResource::OutputBytes, 4),
        );
    assert_eq!(limits.max_output_bytes(), Some(4));
}
