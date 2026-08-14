// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Verifies value transaction commit and rollback semantics.

use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonValueLimits;

/// Verifies only an explicit commit publishes staged value usage.
#[test]
fn test_value_transaction_commit_publishes_usage() {
    let mut budget = JsonValueLimits::empty().with_max_nodes(1).budget();
    let mut transaction = budget.transaction();
    transaction
        .try_admit(JsonMeasurement::Null { depth: 1 })
        .expect("value admission fits");
    assert_eq!(transaction.used_nodes(), Some(1));

    transaction.commit();

    assert_eq!(budget.used_nodes(), Some(1));
}
