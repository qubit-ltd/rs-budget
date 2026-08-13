// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;

/// Verifies that JSON value limits use machine-sized quantities by default.
#[test]
fn test_default_uses_usize_quantity() {
    let _: JsonValueLimits = JsonValueLimits::default();
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

    budget.enter_node_usize(0).expect("one node fits");
    assert_eq!(budget.structure_budget().used_nodes(), 1);
}
