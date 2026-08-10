// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! Tests for nested JSON preflight seeds.

use qubit_budget::BudgetError;
use qubit_budget::JsonLimits;
use qubit_budget::JsonResource;
use qubit_budget::JsonSerdeError;
use qubit_budget::from_slice_with_budget;
use serde_json::Value;

/// Verifies that nested values use the child seed's incremented depth.
#[test]
fn test_json_preflight_child_seed_charges_nested_depth() {
    let mut budget = JsonLimits::new().with_max_depth(1).budget();
    let error = from_slice_with_budget::<Value, _>(b"[null]", &mut budget)
        .expect_err("the nested value should exceed the depth budget");

    assert!(matches!(
        error,
        JsonSerdeError::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::Depth,
            actual: 2,
            maximum: 1,
        })
    ));
}
