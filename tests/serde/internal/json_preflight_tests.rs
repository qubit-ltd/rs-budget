// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! Tests for the private JSON preflight walker.

use qubit_budget::BudgetError;
use qubit_budget::JsonLimits;
use qubit_budget::JsonResource;
use qubit_budget::JsonSerdeError;
use qubit_budget::from_slice_with_budget;
use serde_json::Value;

/// Verifies that the preflight walker charges the root and child nodes.
#[test]
fn test_json_preflight_charges_root_and_child_nodes() {
    let mut budget = JsonLimits::new().with_max_nodes(1).budget();
    let error =
        from_slice_with_budget::<Value, _>(br#"{"value":true}"#, &mut budget)
            .expect_err("the child node should exceed the node budget");

    assert!(matches!(
        error,
        JsonSerdeError::Budget(BudgetError::Insufficient {
            resource: JsonResource::Nodes,
            limit: 1,
            remaining: 0,
            requested: 1,
        })
    ));
}
