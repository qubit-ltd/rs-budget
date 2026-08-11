// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! Tests for lexical JSON preflight depth accounting.

use qubit_budget::BudgetError;
use qubit_budget::JsonLimits;
use qubit_budget::JsonResource;
use qubit_budget::JsonSerdeError;
use qubit_budget::from_slice_with_budget;
use serde_json::Value;

/// Verifies that nested values use root-inclusive lexical depth.
#[test]
fn test_json_lexical_preflight_charges_nested_depth() {
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
