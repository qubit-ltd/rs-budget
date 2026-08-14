// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::BudgetError;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::ResourceLimit;
use qubit_budget::StructureLimits;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueBudget;
use qubit_budget::json::JsonValueLimits;

fn limits_with_all_value_dimensions() -> JsonValueLimits {
    JsonValueLimits::empty()
        .with_structure_limits(
            StructureLimits::empty()
                .with_depth_limit(ResourceLimit::new(
                    JsonResource::Depth,
                    2_usize,
                ))
                .with_nodes_limit(ResourceLimit::new(
                    JsonResource::Nodes,
                    4_usize,
                ))
                .with_sequence_items_limit(ResourceLimit::new(
                    JsonResource::SequenceItems,
                    3_usize,
                ))
                .with_map_entries_limit(ResourceLimit::new(
                    JsonResource::MapEntries,
                    2_usize,
                ))
                .with_key_bytes_limit(ResourceLimit::new(
                    JsonResource::KeyBytes,
                    4_usize,
                )),
        )
        .with_string_bytes_limit(ResourceLimit::new(
            JsonResource::StringBytes,
            3_usize,
        ))
        .with_number_bytes_limit(ResourceLimit::new(
            JsonResource::NumberBytes,
            2_usize,
        ))
        .with_payload_bytes_limit(ResourceLimit::new(
            JsonResource::PayloadBytes,
            8_usize,
        ))
}

/// Verifies that a rejected scalar does not consume its node or payload quota.
#[test]
fn test_enter_string_usize_rejection_is_atomic() {
    let limits =
        JsonValueLimits::empty()
            .with_structure_limits(StructureLimits::empty().with_nodes_limit(
                ResourceLimit::new(JsonResource::Nodes, 1_usize),
            ))
            .with_string_bytes_limit(ResourceLimit::new(
                JsonResource::StringBytes,
                2_usize,
            ))
            .with_payload_bytes_limit(ResourceLimit::new(
                JsonResource::PayloadBytes,
                2_usize,
            ));
    let mut budget = JsonValueBudget::new(limits);

    assert!(budget.enter_string_usize(1, 3).is_err());
    assert_eq!(budget.structure_budget().used_nodes(), 0);
    assert_eq!(
        budget
            .payload_budget()
            .expect("payload is configured")
            .used(),
        0
    );
}

/// Verifies depth inspection does not consume the structural node allowance.
#[test]
fn test_check_depth_does_not_charge_nodes() {
    let limits = JsonValueLimits::empty().with_structure_limits(
        StructureLimits::empty()
            .with_depth_limit(ResourceLimit::new(JsonResource::Depth, 1_usize))
            .with_nodes_limit(ResourceLimit::new(JsonResource::Nodes, 1_usize)),
    );
    let budget = JsonValueBudget::new(limits);

    assert!(budget.check_depth(1).is_ok());
    assert!(budget.check_depth(2).is_err());
    assert_eq!(budget.structure_budget().used_nodes(), 0);
}

/// Verifies that successful array and object admissions charge exactly one
/// node.
#[test]
fn test_enter_array_and_object_charge_nodes() {
    let mut budget = JsonValueBudget::new(limits_with_all_value_dimensions());

    budget.enter_array(1, 3).expect("array fits");
    budget.enter_object(2, 2).expect("object fits");

    assert_eq!(budget.structure_budget().used_nodes(), 2);
}

/// Verifies that rejected container admissions leave node accounting unchanged.
#[test]
fn test_enter_containers_reject_without_charging_nodes() {
    let limits = JsonValueLimits::empty().with_structure_limits(
        StructureLimits::empty()
            .with_nodes_limit(ResourceLimit::new(JsonResource::Nodes, 1_usize))
            .with_sequence_items_limit(ResourceLimit::new(
                JsonResource::SequenceItems,
                1_usize,
            ))
            .with_map_entries_limit(ResourceLimit::new(
                JsonResource::MapEntries,
                1_usize,
            )),
    );
    let mut budget = JsonValueBudget::new(limits);

    assert!(budget.enter_array(1, 2).is_err());
    assert_eq!(budget.structure_budget().used_nodes(), 0);
    assert!(budget.enter_object(1, 2).is_err());
    assert_eq!(budget.structure_budget().used_nodes(), 0);
}

/// Verifies that scalar admissions charge both node and cumulative payload.
#[test]
fn test_enter_string_and_number_charge_node_and_payload() {
    let mut budget = JsonValueBudget::new(limits_with_all_value_dimensions());

    budget.enter_string(1, 3).expect("string fits");
    budget.enter_number(1, 2).expect("number fits");

    assert_eq!(budget.structure_budget().used_nodes(), 2);
    assert_eq!(
        budget.payload_budget().expect("payload configured").used(),
        5
    );
}

/// Verifies the documented node, point-limit, then payload error priority.
#[test]
fn test_scalar_error_priority_is_node_then_point_then_payload() {
    let mut node_limited = JsonValueBudget::new(
        JsonValueLimits::empty()
            .with_structure_limits(StructureLimits::empty().with_nodes_limit(
                ResourceLimit::new(JsonResource::Nodes, 0_usize),
            ))
            .with_string_bytes_limit(ResourceLimit::new(
                JsonResource::StringBytes,
                1_usize,
            ))
            .with_payload_bytes_limit(ResourceLimit::new(
                JsonResource::PayloadBytes,
                1_usize,
            )),
    );
    assert!(matches!(
        node_limited.enter_string(1, 2),
        Err(BudgetError::Insufficient {
            resource: JsonResource::Nodes,
            ..
        })
    ));

    let mut point_limited = JsonValueBudget::new(
        JsonValueLimits::empty()
            .with_structure_limits(StructureLimits::empty().with_nodes_limit(
                ResourceLimit::new(JsonResource::Nodes, 1_usize),
            ))
            .with_string_bytes_limit(ResourceLimit::new(
                JsonResource::StringBytes,
                1_usize,
            ))
            .with_payload_bytes_limit(ResourceLimit::new(
                JsonResource::PayloadBytes,
                3_usize,
            )),
    );
    assert!(matches!(
        point_limited.enter_string(1, 2),
        Err(BudgetError::LimitExceeded {
            resource: JsonResource::StringBytes,
            ..
        })
    ));

    let mut payload_limited = JsonValueBudget::new(
        JsonValueLimits::empty()
            .with_structure_limits(StructureLimits::empty().with_nodes_limit(
                ResourceLimit::new(JsonResource::Nodes, 1_usize),
            ))
            .with_string_bytes_limit(ResourceLimit::new(
                JsonResource::StringBytes,
                3_usize,
            ))
            .with_payload_bytes_limit(ResourceLimit::new(
                JsonResource::PayloadBytes,
                1_usize,
            )),
    );
    assert!(matches!(
        payload_limited.enter_string(1, 2),
        Err(BudgetError::Insufficient {
            resource: JsonResource::PayloadBytes,
            ..
        })
    ));
}

/// Verifies that native conversions report narrow quantity failures and skip
/// conversion when the corresponding dimension is unconfigured.
#[test]
fn test_native_conversion_respects_configured_dimensions() {
    let mut unconfigured = JsonValueBudget::<JsonResource, u8>::new(
        JsonValueLimits::unconfigured(),
    );
    unconfigured
        .enter_string_usize(usize::MAX, usize::MAX)
        .expect("unconfigured dimensions do not require conversion");
    assert_eq!(unconfigured.structure_budget().used_nodes(), 0);

    let limits = JsonValueLimits::<JsonResource, u8>::unconfigured()
        .with_structure_limits(
            StructureLimits::empty()
                .with_depth_limit(ResourceLimit::new(
                    JsonResource::Depth,
                    u8::MAX,
                ))
                .with_nodes_limit(ResourceLimit::new(
                    JsonResource::Nodes,
                    u8::MAX,
                )),
        );
    let mut narrow = JsonValueBudget::new(limits);
    assert!(matches!(
        narrow.enter_node_usize(usize::from(u8::MAX) + 1),
        Err(MeasuredBudgetError::Quantity {
            resource: JsonResource::Depth,
            ..
        })
    ));
}

/// Verifies that reset restores all counters while retaining the configured
/// limits.
#[test]
fn test_reset_restores_original_limits_and_usage() {
    let limits = limits_with_all_value_dimensions();
    let mut budget = JsonValueBudget::new(limits);
    budget.enter_string(1, 3).expect("string fits");

    budget.reset();

    assert_eq!(budget.structure_budget().used_nodes(), 0);
    assert_eq!(
        budget.payload_budget().expect("payload configured").used(),
        0
    );
    assert_eq!(budget.limits().max_string_bytes(), Some(3));
    budget
        .enter_number(1, 2)
        .expect("reset budget accepts another value");
}

/// Verifies the explicit check and consume helpers for every payload and
/// structural dimension.
#[test]
fn test_explicit_dimension_helpers_preserve_accounting_semantics() {
    let mut budget = JsonValueBudget::new(limits_with_all_value_dimensions());

    budget.check_sequence_items(2).expect("sequence fits");
    budget.check_sequence_items_usize(2).expect("sequence fits");
    budget.check_map_entries(2).expect("map fits");
    budget.check_map_entries_usize(2).expect("map fits");
    budget.check_key_bytes(3).expect("key fits");
    budget.check_key_bytes_usize(3).expect("key fits");
    budget.check_string_bytes(3).expect("string fits");
    budget.check_string_bytes_usize(3).expect("string fits");
    budget.check_number_bytes(2).expect("number fits");
    budget.check_number_bytes_usize(2).expect("number fits");

    budget.consume_key_bytes(1).expect("key payload fits");
    budget.consume_key_bytes_usize(1).expect("key payload fits");
    budget.consume_string_bytes(1).expect("string payload fits");
    budget
        .consume_string_bytes_usize(1)
        .expect("string payload fits");
    budget.consume_number_bytes(1).expect("number payload fits");
    budget
        .consume_number_bytes_usize(1)
        .expect("number payload fits");
    budget.charge_node().expect("explicit node charge fits");
    assert_eq!(budget.structure_budget().used_nodes(), 1);
    assert_eq!(budget.payload_budget().expect("payload").used(), 6);
}

/// Verifies native conversion and direct generic entry points for narrow
/// quantities.
#[test]
fn test_native_entry_helpers_cover_arrays_objects_strings_and_numbers() {
    let limits = limits_with_all_value_dimensions();
    let mut budget = JsonValueBudget::new(limits);
    budget.enter_array_usize(1, 1).expect("array fits");
    budget.enter_object_usize(1, 1).expect("object fits");
    budget.enter_number_usize(1, 1).expect("number fits");

    let mut generic = JsonValueBudget::new(limits_with_all_value_dimensions());
    generic.enter_array(1, 1).expect("array fits");
    generic.enter_object(1, 1).expect("object fits");
    generic.enter_string(1, 1).expect("string fits");
    generic.enter_number(1, 1).expect("number fits");
}
