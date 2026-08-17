// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
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
    let limits = JsonEncodeLimits::<JsonResource, usize>::builder()
        .max_output_bytes(1)
        .max_depth(2)
        .max_nodes(3)
        .max_sequence_items(4)
        .max_map_entries(5)
        .max_key_bytes(6)
        .max_string_bytes(7)
        .max_number_bytes(8)
        .max_payload_bytes(9)
        .build();

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
    let limits = JsonEncodeLimits::<JsonResource, usize>::builder()
        .max_depth(2)
        .build();
    let _: &JsonValueLimits = limits.value_limits();
    assert_eq!(limits.value_limits().max_depth(), Some(2));
    assert_eq!(limits.into_value_limits().max_depth(), Some(2));
}

#[test]
fn test_empty_encode_limits_report_unconfigured_maximum() {
    assert_eq!(
        JsonEncodeLimits::<JsonResource, usize>::builder()
            .build()
            .max_output_bytes(),
        None
    );
}

#[test]
fn test_generic_encode_limits_expose_maximum() {
    let limits = JsonEncodeLimits::<JsonResource, u8>::builder()
        .max_output_bytes(4)
        .max_depth(5)
        .max_nodes(6)
        .max_sequence_items(7)
        .max_map_entries(8)
        .max_key_bytes(9)
        .max_string_bytes(10)
        .max_number_bytes(11)
        .max_payload_bytes(12)
        .build();
    assert_eq!(limits.max_output_bytes(), Some(4));
    assert_eq!(limits.value_limits().max_payload_bytes(), Some(12));
}
