// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for JSON budget enforcement.

use qubit_budget::BudgetError;
use qubit_budget::JsonLimits;
use qubit_budget::JsonResource;

#[test]
fn test_json_string_and_number_bytes_are_point_limits() {
    let budget = JsonLimits::new()
        .with_max_string_bytes(2)
        .with_max_number_bytes(3)
        .budget();

    budget
        .check_string_bytes(2)
        .expect("first string should fit");
    budget
        .check_string_bytes(2)
        .expect("strings must not accumulate");
    budget
        .check_number_bytes(3)
        .expect("first number should fit");
    budget
        .check_number_bytes(3)
        .expect("numbers must not accumulate");
}

#[test]
fn test_json_budget_checks_complete_input_at_the_exact_limit() {
    let budget = JsonLimits::new().with_max_input_bytes(3).budget();

    budget
        .check_input_bytes(3)
        .expect("complete input at the limit should fit");
    assert!(matches!(
        budget.check_input_bytes(4),
        Err(BudgetError::LimitExceeded {
            resource: JsonResource::InputBytes,
            actual: 4,
            maximum: 3,
        })
    ));
}

#[test]
fn test_json_budget_checks_root_inclusive_depth() {
    let budget = JsonLimits::new().with_max_depth(1).budget();

    budget.check_depth(1).expect("the root depth should fit");
    assert!(matches!(
        budget.check_depth(2),
        Err(BudgetError::LimitExceeded {
            resource: JsonResource::Depth,
            actual: 2,
            maximum: 1,
        })
    ));
}

#[test]
fn test_json_budget_checks_sequence_and_map_sizes_independently() {
    let budget = JsonLimits::new()
        .with_max_sequence_items(2)
        .with_max_map_entries(3)
        .budget();

    budget
        .check_sequence_items(2)
        .expect("first sequence at the limit should fit");
    budget
        .check_sequence_items(2)
        .expect("sequence checks must not accumulate");
    budget
        .check_map_entries(3)
        .expect("first map at the limit should fit");
    budget
        .check_map_entries(3)
        .expect("map checks must not accumulate");
}

#[test]
fn test_json_budget_node_charge_is_cumulative_and_atomic() {
    let mut budget = JsonLimits::new().with_max_nodes(2).budget();

    budget.charge_node().expect("first node should fit");
    budget.charge_node().expect("second node should fit");
    assert!(matches!(
        budget.charge_node(),
        Err(BudgetError::Insufficient {
            resource: JsonResource::Nodes,
            limit: 2,
            remaining: 0,
            requested: 1,
        })
    ));
    assert!(matches!(
        budget.charge_node(),
        Err(BudgetError::Insufficient {
            resource: JsonResource::Nodes,
            limit: 2,
            remaining: 0,
            requested: 1,
        })
    ));
}

#[test]
fn test_unconfigured_json_limits_allow_all_checks() {
    let mut budget = JsonLimits::new().budget();

    budget
        .check_input_bytes(usize::MAX)
        .expect("input size is unconfigured");
    budget
        .check_depth(usize::MAX)
        .expect("depth is unconfigured");
    budget.charge_node().expect("node count is unconfigured");
    budget
        .check_sequence_items(usize::MAX)
        .expect("sequence size is unconfigured");
    budget
        .check_map_entries(usize::MAX)
        .expect("map size is unconfigured");
    budget
        .check_string_bytes(usize::MAX)
        .expect("string bytes are unconfigured");
    budget
        .check_number_bytes(usize::MAX)
        .expect("number bytes are unconfigured");
}

#[test]
fn test_json_budget_delegates_structure_and_output_key_limits() {
    let mut budget = JsonLimits::new()
        .with_max_depth(1)
        .with_max_key_bytes(2)
        .with_max_output_bytes(3)
        .with_max_nodes(2)
        .budget();

    budget.check_depth(1).expect("depth should fit");
    budget.check_key_bytes(2).expect("key bytes should fit");
    budget
        .check_output_bytes(3)
        .expect("output bytes should fit");
    budget.charge_node().expect("first node should fit");
    budget.charge_nodes(1).expect("second node should fit");
    assert!(matches!(
        budget.check_output_bytes(4),
        Err(BudgetError::LimitExceeded {
            resource: JsonResource::OutputBytes,
            actual: 4,
            maximum: 3,
        })
    ));
}

/// Verifies that container entry points share limits and node accounting.
#[test]
fn test_json_budget_enters_containers_and_exposes_shared_limits() {
    let limits = JsonLimits::new()
        .with_max_depth(2)
        .with_max_nodes(2)
        .with_max_sequence_items(1)
        .with_max_map_entries(1);
    let mut budget = limits.budget();

    budget
        .enter_array(1, 1)
        .expect("the array entry should fit");
    budget
        .enter_object(1, 1)
        .expect("the object entry should fit");
    assert_eq!(budget.limits(), &limits);
    assert_eq!(
        budget.structure_budget().limits(),
        &limits.structure_limits()
    );
}
