// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! Tests for JSON/Serde adapter error propagation.

use std::error::Error;

use qubit_budget::BudgetError;
use qubit_budget::JsonResource;
use qubit_budget::JsonSerdeError;

#[test]
fn test_budget_error_is_the_direct_source() {
    let error = JsonSerdeError::Budget(BudgetError::LimitExceeded {
        resource: JsonResource::StringBytes,
        actual: 4,
        maximum: 3,
    });

    let source = error.source().expect("budget error should be the source");
    assert!(source.to_string().contains("StringBytes"));
}
