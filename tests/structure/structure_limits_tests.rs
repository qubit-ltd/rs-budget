// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for structural limit configuration.

use qubit_budget::BudgetError;
use qubit_budget::Observation;
use qubit_budget::ResourceLimit;
use qubit_budget::StructureBudget;
use qubit_budget::StructureLimits;
use qubit_budget::StructureResource;

#[test]
fn test_default_structure_limits_match_default_structure_budget_quantity() {
    let limits = StructureLimits::new();
    let budget: StructureBudget = limits.budget();

    assert_eq!(budget.used_nodes(), 0_usize);
}

#[test]
fn test_structure_limits_expose_configured_values() {
    let limits = StructureLimits::new()
        .with_max_depth(1)
        .with_max_nodes(2)
        .with_max_sequence_items(3)
        .with_max_map_entries(4)
        .with_max_key_bytes(5);

    assert_eq!(limits.max_depth(), Some(1));
    assert_eq!(limits.max_nodes(), Some(2));
    assert_eq!(limits.max_sequence_items(), Some(3));
    assert_eq!(limits.max_map_entries(), Some(4));
    assert_eq!(limits.max_key_bytes(), Some(5));
}

/// Verifies custom-resource structural limits use the generic key setter.
#[test]
fn test_generic_structure_limits_support_custom_key_limit() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Resource {
        KeyBytes,
    }

    let limits = StructureLimits::<Resource, u8>::empty()
        .with_key_bytes_limit(ResourceLimit::new(Resource::KeyBytes, 3));

    assert_eq!(limits.max_key_bytes(), Some(3));
    assert_eq!(
        limits.key_bytes_limit().unwrap().resource(),
        &Resource::KeyBytes
    );
}

#[test]
fn test_with_max_methods_bind_each_limit_to_its_structure_resource() {
    let mut budget = StructureLimits::new()
        .with_max_depth(1)
        .with_max_nodes(1)
        .with_max_sequence_items(1)
        .with_max_map_entries(1)
        .budget();

    assert!(matches!(
        budget.check_depth(2),
        Err(BudgetError::LimitExceeded {
            resource: StructureResource::Depth,
            observed: Observation::Exact(2),
            maximum: 1,
        })
    ));
    assert!(matches!(budget.charge_node(), Ok(())));
    assert!(matches!(
        budget.charge_node(),
        Err(BudgetError::Insufficient {
            resource: StructureResource::Nodes,
            limit: 1,
            remaining: 0,
            requested: 1,
        })
    ));
    assert!(matches!(
        budget.check_sequence_items(2),
        Err(BudgetError::LimitExceeded {
            resource: StructureResource::SequenceItems,
            observed: Observation::Exact(2),
            maximum: 1,
        })
    ));
    assert!(matches!(
        budget.check_map_entries(2),
        Err(BudgetError::LimitExceeded {
            resource: StructureResource::MapEntries,
            observed: Observation::Exact(2),
            maximum: 1,
        })
    ));
}
