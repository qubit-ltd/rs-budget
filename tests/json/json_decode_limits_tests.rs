// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::json::JsonDecodeLimits;

/// Verifies that JSON decode limits use machine-sized quantities by default.
#[test]
fn test_default_uses_usize_quantity() {
    let _: JsonDecodeLimits = JsonDecodeLimits::default();
}

/// Verifies that decode builders configure input and nested value limits.
#[test]
fn test_standard_builder_configures_decode_dimensions() {
    let limits = JsonDecodeLimits::empty()
        .with_max_input_bytes(1)
        .with_max_normalized_input_bytes(2)
        .with_max_depth(3)
        .with_max_nodes(4)
        .with_max_sequence_items(5)
        .with_max_map_entries(6)
        .with_max_key_bytes(7)
        .with_max_string_bytes(8)
        .with_max_number_bytes(9)
        .with_max_payload_bytes(10);

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
