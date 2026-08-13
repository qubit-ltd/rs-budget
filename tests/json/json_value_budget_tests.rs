// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::ResourceLimit;
use qubit_budget::StructureLimits;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueBudget;
use qubit_budget::json::JsonValueLimits;

/// Verifies that a rejected scalar does not consume its node or payload quota.
#[test]
fn test_enter_string_usize_rejection_is_atomic() {
    let limits = JsonValueLimits::empty()
        .with_structure_limits(
            StructureLimits::empty()
                .with_nodes_limit(ResourceLimit::new(JsonResource::Nodes, 1_usize)),
        )
        .with_string_bytes_limit(ResourceLimit::new(JsonResource::StringBytes, 2_usize))
        .with_payload_bytes_limit(ResourceLimit::new(JsonResource::PayloadBytes, 2_usize));
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
