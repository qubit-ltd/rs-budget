// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! Tests for the private JSON preflight visitor.

use qubit_budget::BudgetError;
use qubit_budget::JsonLimits;
use qubit_budget::JsonResource;
use qubit_budget::JsonSerdeError;
use qubit_budget::from_slice_with_budget;

/// Verifies that the visitor charges the UTF-8 length of string values.
#[test]
fn test_json_preflight_visitor_charges_string_bytes() {
    let mut budget = JsonLimits::new().with_max_string_bytes(3).budget();
    let error = from_slice_with_budget::<String, _>(br#""hello""#, &mut budget)
        .expect_err("the string should exceed the byte budget");

    assert!(matches!(
        error,
        JsonSerdeError::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::StringBytes,
            actual: 5,
            maximum: 3,
        })
    ));
}
