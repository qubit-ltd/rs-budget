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
use qubit_budget::StructureLimits;
use qubit_budget::StructureResource;

#[test]
fn test_structure_limits_expose_configured_values() {
    let limits = StructureLimits::<StructureResource, usize>::builder()
        .max_depth(1)
        .max_nodes(2)
        .max_sequence_items(3)
        .max_map_entries(4)
        .max_key_bytes(5)
        .build();

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

    let limits = StructureLimits::<Resource, u8>::builder()
        .key_bytes_limit(ResourceLimit::new(Resource::KeyBytes, 3))
        .build();

    assert_eq!(limits.max_key_bytes(), Some(3));
    assert_eq!(limits.key_bytes_limit().unwrap().resource(), &Resource::KeyBytes);
}

#[test]
fn test_builder_max_methods_bind_each_limit_to_its_structure_resource() {
    let mut budget = StructureLimits::<StructureResource, usize>::builder()
        .max_depth(1)
        .max_nodes(1)
        .max_sequence_items(1)
        .max_map_entries(1)
        .build()
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

#[test]
fn test_to_builder_clones_non_copy_limits_without_consuming_the_original() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct OwnedResource(String);

    let depth = OwnedResource("depth".to_owned());
    let nodes = OwnedResource("nodes".to_owned());
    let limits = StructureLimits::<OwnedResource, usize>::builder()
        .depth_limit(ResourceLimit::new(depth.clone(), 2))
        .build();

    let rebuilt = limits
        .to_builder()
        .nodes_limit(ResourceLimit::new(nodes.clone(), 3))
        .build();

    assert_eq!(limits.depth_limit().unwrap().resource(), &depth);
    assert_eq!(limits.max_depth(), Some(2));
    assert_eq!(limits.max_nodes(), None);
    assert_eq!(rebuilt.depth_limit().unwrap().resource(), &depth);
    assert_eq!(rebuilt.nodes_limit().unwrap().resource(), &nodes);
    assert_eq!(rebuilt.max_depth(), Some(2));
    assert_eq!(rebuilt.max_nodes(), Some(3));
}
