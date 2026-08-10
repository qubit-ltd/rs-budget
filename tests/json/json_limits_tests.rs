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

#[test]
fn test_with_max_methods_bind_each_limit_to_its_json_resource() {
    let mut budget = JsonLimits::new()
        .with_max_input_bytes(1)
        .with_max_depth(1)
        .with_max_nodes(1)
        .with_max_sequence_items(1)
        .with_max_map_entries(1)
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
