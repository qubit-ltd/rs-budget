// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for JSON limit configuration.

use qubit_budget::BudgetError;
use qubit_budget::JsonLimits;
use qubit_budget::JsonResource;
use qubit_budget::StructureLimits;

#[test]
fn test_json_limits_expose_configured_values() {
    let limits = JsonLimits::new()
        .with_max_depth(1)
        .with_max_nodes(2)
        .with_max_sequence_items(3)
        .with_max_map_entries(4);

    assert_eq!(limits.max_depth(), Some(1));
    assert_eq!(limits.max_nodes(), Some(2));
    assert_eq!(limits.max_sequence_items(), Some(3));
    assert_eq!(limits.max_map_entries(), Some(4));
}

#[test]
fn test_json_limits_compose_structure_limits() {
    let structure_limits = StructureLimits::new()
        .with_max_depth(1)
        .with_max_nodes(2)
        .with_max_sequence_items(3)
        .with_max_map_entries(4);
    let limits = JsonLimits::new().with_structure_limits(structure_limits);

    let converted = StructureLimits::<JsonResource, usize>::default()
        .with_depth_limit(qubit_budget::ResourceLimit::new(JsonResource::Depth, 1))
        .with_nodes_limit(qubit_budget::ResourceLimit::new(JsonResource::Nodes, 2))
        .with_sequence_items_limit(qubit_budget::ResourceLimit::new(
            JsonResource::SequenceItems,
            3,
        ))
        .with_map_entries_limit(qubit_budget::ResourceLimit::new(
            JsonResource::MapEntries,
            4,
        ));
    assert_eq!(limits.structure_limits(), converted);
}

#[test]
fn test_with_max_methods_bind_each_limit_to_its_json_resource() {
    let mut budget = JsonLimits::new()
        .with_max_input_bytes(1)
        .with_max_output_bytes(1)
        .with_max_depth(1)
        .with_max_nodes(1)
        .with_max_sequence_items(1)
        .with_max_map_entries(1)
        .with_max_key_bytes(1)
        .with_max_string_bytes(1)
        .with_max_number_bytes(1)
        .budget();

    assert!(matches!(
        budget.check_input_bytes(2),
        Err(BudgetError::LimitExceeded {
            resource: JsonResource::InputBytes,
            actual: 2,
            maximum: 1
        })
    ));
    assert!(matches!(
        budget.check_output_bytes(2),
        Err(BudgetError::LimitExceeded {
            resource: JsonResource::OutputBytes,
            actual: 2,
            maximum: 1
        })
    ));
    assert!(matches!(
        budget.check_depth(2),
        Err(BudgetError::LimitExceeded {
            resource: JsonResource::Depth,
            actual: 2,
            maximum: 1
        })
    ));
    assert!(matches!(budget.charge_node(), Ok(())));
    assert!(matches!(
        budget.charge_node(),
        Err(BudgetError::Insufficient {
            resource: JsonResource::Nodes,
            limit: 1,
            remaining: 0,
            requested: 1
        })
    ));
    assert!(matches!(
        budget.check_sequence_items(2),
        Err(BudgetError::LimitExceeded {
            resource: JsonResource::SequenceItems,
            actual: 2,
            maximum: 1
        })
    ));
    assert!(matches!(
        budget.check_map_entries(2),
        Err(BudgetError::LimitExceeded {
            resource: JsonResource::MapEntries,
            actual: 2,
            maximum: 1
        })
    ));
    assert!(matches!(
        budget.check_key_bytes(2),
        Err(BudgetError::LimitExceeded {
            resource: JsonResource::KeyBytes,
            actual: 2,
            maximum: 1
        })
    ));
    assert!(matches!(
        budget.check_string_bytes(2),
        Err(BudgetError::LimitExceeded {
            resource: JsonResource::StringBytes,
            actual: 2,
            maximum: 1
        })
    ));
    assert!(matches!(
        budget.check_number_bytes(2),
        Err(BudgetError::LimitExceeded {
            resource: JsonResource::NumberBytes,
            actual: 2,
            maximum: 1
        })
    ));
}
