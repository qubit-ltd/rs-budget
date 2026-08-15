// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Verifies value transaction commit and rollback semantics.

use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;
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

/// Verifies that checking the next sequence item does not mutate accounting.
#[test]
fn test_check_sequence_items_rejects_next_item_without_mutation() {
    let mut budget = JsonValueLimits::<JsonResource, usize>::empty()
        .with_max_sequence_items(1)
        .with_max_nodes(4)
        .budget();
    let transaction = budget.transaction();

    let _ = transaction
        .check_sequence_items(2)
        .expect_err("the next item exceeds the configured limit");
    assert_eq!(transaction.used_nodes(), Some(0));
    assert_eq!(transaction.remaining_nodes(), Some(4));
    transaction.commit();
    assert_eq!(budget.used_nodes(), Some(0));
}
