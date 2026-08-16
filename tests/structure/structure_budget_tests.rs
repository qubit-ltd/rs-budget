// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for structural budget enforcement.

use qubit_budget::BudgetError;
use qubit_budget::ResourceLimit;
use qubit_budget::StructureLimits;
use qubit_budget::StructureResource;

#[test]
fn test_structure_budget_distinguishes_point_and_cumulative_limits() {
    let limits = StructureLimits::<StructureResource, usize>::builder()
        .max_depth(2)
        .max_nodes(2)
        .max_sequence_items(1)
        .max_map_entries(1)
        .build();
    let mut budget = limits.budget();

    budget.check_depth(2).expect("exact depth should fit");
    budget
        .check_sequence_items(1)
        .expect("first sequence should fit");
    budget
        .check_sequence_items(1)
        .expect("point checks must not accumulate");
    budget.charge_node().expect("first node should fit");
    budget.charge_node().expect("second node should fit");
    assert!(matches!(
        budget.charge_node(),
        Err(BudgetError::Insufficient {
            resource: StructureResource::Nodes,
            limit: 2,
            remaining: 0,
            requested: 1,
        })
    ));
}

#[test]
fn test_unconfigured_structure_limits_allow_all_checks() {
    let mut budget = StructureLimits::<StructureResource, usize>::builder()
        .build()
        .budget();

    budget
        .check_depth(usize::MAX)
        .expect("depth is unconfigured");
    budget
        .check_sequence_items(usize::MAX)
        .expect("sequence size is unconfigured");
    budget
        .check_map_entries(usize::MAX)
        .expect("map size is unconfigured");
    budget.charge_node().expect("node count is unconfigured");
}

#[test]
fn test_charge_node_failure_is_atomic() {
    let mut budget = StructureLimits::<StructureResource, usize>::builder()
        .max_nodes(1)
        .build()
        .budget();

    budget.charge_node().expect("first node should fit");
    assert!(matches!(
        budget.charge_node(),
        Err(BudgetError::Insufficient { .. })
    ));
    assert!(matches!(
        budget.charge_node(),
        Err(BudgetError::Insufficient {
            resource: StructureResource::Nodes,
            limit: 1,
            remaining: 0,
            requested: 1,
        })
    ));
}

#[test]
fn test_budget_creates_independent_node_accounting_sessions() {
    let limits = StructureLimits::<StructureResource, usize>::builder()
        .max_nodes(1)
        .build();
    let mut first_budget = limits.budget();
    let mut second_budget = limits.budget();

    first_budget
        .charge_node()
        .expect("first session should fit");
    assert!(first_budget.charge_node().is_err());
    second_budget
        .charge_node()
        .expect("second session must start with a fresh node budget");
}

#[test]
fn test_generic_structure_budget_uses_custom_resource_and_quantity() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Resource {
        Nodes,
    }

    let limits = StructureLimits::<Resource, u8>::builder()
        .nodes_limit(ResourceLimit::new(Resource::Nodes, 2))
        .build();
    let mut budget = limits.budget();
    budget.charge_node().expect("first node should fit");
    budget.charge_node().expect("second node should fit");
    assert!(matches!(
        budget.charge_node(),
        Err(BudgetError::Insufficient {
            resource: Resource::Nodes,
            limit: 2,
            remaining: 0,
            requested: 1,
        })
    ));
}

/// Verifies sequence and map entry points perform all atomic checks.
#[test]
fn test_structure_budget_enters_sequences_and_maps() {
    let limits = StructureLimits::<StructureResource, usize>::builder()
        .max_depth(2)
        .max_nodes(2)
        .max_sequence_items(1)
        .max_map_entries(1)
        .build();
    let mut budget = limits.budget();

    budget
        .enter_sequence(1, 1)
        .expect("the sequence entry should fit");
    budget.enter_map(1, 1).expect("the map entry should fit");
    assert_eq!(budget.limits(), &limits);
}

/// Verifies depth, sequence, and map entry failures do not charge nodes.
#[test]
fn test_enter_node_depth_failure_does_not_charge_nodes() {
    let limits = StructureLimits::<StructureResource, usize>::builder()
        .max_depth(1)
        .max_nodes(1)
        .build();
    let mut budget = limits.budget();

    assert!(matches!(
        budget.enter_node(2),
        Err(BudgetError::LimitExceeded { .. })
    ));
    assert_eq!(budget.used_nodes(), 0);
}

/// Verifies sequence item failures do not charge nodes after depth passes.
#[test]
fn test_enter_sequence_items_failure_does_not_charge_nodes() {
    let limits = StructureLimits::<StructureResource, usize>::builder()
        .max_depth(2)
        .max_sequence_items(1)
        .max_nodes(1)
        .build();
    let mut budget = limits.budget();

    assert!(matches!(
        budget.enter_sequence(1, 2),
        Err(BudgetError::LimitExceeded { .. })
    ));
    assert_eq!(budget.used_nodes(), 0);
}

/// Verifies map entry failures do not charge nodes after depth passes.
#[test]
fn test_enter_map_entries_failure_does_not_charge_nodes() {
    let limits = StructureLimits::<StructureResource, usize>::builder()
        .max_depth(2)
        .max_map_entries(1)
        .max_nodes(1)
        .build();
    let mut budget = limits.budget();

    assert!(matches!(
        budget.enter_map(1, 2),
        Err(BudgetError::LimitExceeded { .. })
    ));
    assert_eq!(budget.used_nodes(), 0);
}

/// Verifies node entry, key checks, and node usage share one budget session.
#[test]
fn test_structure_budget_tracks_nodes_and_checks_key_bytes() {
    let limits = StructureLimits::<StructureResource, usize>::builder()
        .max_nodes(1)
        .max_key_bytes(2)
        .build();
    let mut budget = limits.budget();

    budget
        .check_key_bytes(2)
        .expect("an exact key byte limit should fit");
    budget
        .enter_node(1)
        .expect("the first node should fit the node budget");

    assert_eq!(budget.used_nodes(), 1);
}
