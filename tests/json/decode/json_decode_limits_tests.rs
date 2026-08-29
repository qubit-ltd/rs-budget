// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;

/// Verifies that JSON decode limits use machine-sized quantities by default.
#[test]
fn test_default_uses_usize_quantity() {
    let _: JsonDecodeLimits = JsonDecodeLimits::default();
}

/// Verifies that decode builders configure input and nested value limits.
#[test]
fn test_standard_builder_configures_decode_dimensions() {
    let limits = JsonDecodeLimits::<JsonResource, usize>::builder()
        .max_input_bytes(1)
        .max_normalized_input_bytes(2)
        .max_depth(3)
        .max_nodes(4)
        .max_sequence_items(5)
        .max_map_entries(6)
        .max_key_bytes(7)
        .max_string_bytes(8)
        .max_number_bytes(9)
        .max_payload_bytes(10)
        .build();

    assert_eq!(limits.max_input_bytes(), Some(1));
    assert_eq!(limits.max_normalized_input_bytes(), Some(2));
    assert_eq!(limits.value_limits().max_depth(), Some(3));
    assert_eq!(limits.value_limits().max_nodes(), Some(4));
    assert_eq!(limits.value_limits().max_sequence_items(), Some(5));
    assert_eq!(limits.value_limits().max_map_entries(), Some(6));
    assert_eq!(limits.value_limits().max_key_bytes(), Some(7));
    assert_eq!(limits.value_limits().max_string_bytes(), Some(8));
    assert_eq!(limits.value_limits().max_number_bytes(), Some(9));
    assert_eq!(limits.value_limits().max_payload_bytes(), Some(10));
}

/// Verifies that nested value limits may be borrowed or explicitly consumed.
#[test]
fn test_value_limits_expresses_borrowing_and_ownership() {
    let limits = JsonDecodeLimits::<JsonResource, usize>::builder().max_depth(3).build();
    let _: &JsonValueLimits = limits.value_limits();
    assert_eq!(limits.value_limits().max_depth(), Some(3));
    assert_eq!(limits.into_value_limits().max_depth(), Some(3));
}

#[test]
fn test_new_decode_limits_report_unconfigured_maxima() {
    let limits = JsonDecodeLimits::<JsonResource, usize>::new();
    assert_eq!(limits.max_input_bytes(), None);
    assert_eq!(limits.max_normalized_input_bytes(), None);
}

#[test]
fn test_generic_decode_limits_expose_maxima() {
    let limits = JsonDecodeLimits::<JsonResource, u8>::builder()
        .max_input_bytes(4)
        .max_normalized_input_bytes(5)
        .max_depth(6)
        .max_nodes(7)
        .max_sequence_items(8)
        .max_map_entries(9)
        .max_key_bytes(10)
        .max_string_bytes(11)
        .max_number_bytes(12)
        .max_payload_bytes(13)
        .build();
    assert_eq!(limits.max_input_bytes(), Some(4));
    assert_eq!(limits.max_normalized_input_bytes(), Some(5));
    assert_eq!(limits.value_limits().max_payload_bytes(), Some(13));
}
