// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Verifies value transaction commit and rollback semantics.

use qubit_budget::json::JsonContainerKind;
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;

/// Verifies only an explicit commit publishes staged value usage.
#[test]
fn test_value_transaction_commit_publishes_usage() {
    let mut budget = JsonValueLimits::<JsonResource, usize>::builder()
        .max_nodes(1)
        .build()
        .budget();
    let mut transaction = budget.transaction();
    transaction
        .try_admit(JsonMeasurement::Null { depth: 1 })
        .expect("value admission fits");
    assert_eq!(transaction.used_nodes(), Some(1));

    transaction.commit().expect("successful transaction commits");

    assert_eq!(budget.used_nodes(), Some(1));
}

/// Verifies prospective container checks retain transaction accounting.
#[test]
fn test_check_container_count_rejects_next_item_without_mutation() {
    let mut budget = JsonValueLimits::<JsonResource, usize>::builder()
        .max_sequence_items(1)
        .max_nodes(4)
        .build()
        .budget();
    let mut transaction = budget.transaction();

    let first_error = transaction
        .check_container_count(JsonContainerKind::Sequence, 2)
        .expect_err("the next item exceeds the configured limit");
    assert_eq!(transaction.used_nodes(), Some(0));
    assert_eq!(transaction.remaining_nodes(), Some(4));
    let repeated_error = transaction
        .check_container_count(JsonContainerKind::Sequence, 1)
        .expect_err("a failed count check poisons the transaction");
    assert_eq!(repeated_error.resource(), first_error.resource());
    let commit_error = transaction.commit().expect_err("poisoned transaction cannot commit");
    assert_eq!(commit_error.resource(), first_error.resource());
    assert_eq!(budget.used_nodes(), Some(0));
}

/// Verifies prospective map checks use the object-entry resource limit.
#[test]
fn test_check_container_count_rejects_next_map_entry() {
    let mut budget = JsonValueLimits::<JsonResource, usize>::builder()
        .max_map_entries(1)
        .build()
        .budget();
    let mut transaction = budget.transaction();

    assert!(transaction.check_container_count(JsonContainerKind::Map, 1).is_ok());
    let error = transaction
        .check_container_count(JsonContainerKind::Map, 2)
        .expect_err("the second map entry exceeds the configured limit");

    assert_eq!(error.resource(), &JsonResource::MapEntries);
}

#[test]
fn test_enter_container_admits_node_before_children() {
    let mut budget = JsonValueLimits::<JsonResource, usize>::builder()
        .max_depth(1)
        .max_nodes(1)
        .build()
        .budget();
    let mut transaction = budget.transaction();

    transaction
        .try_enter_container(JsonContainerKind::Sequence, 1)
        .expect("container admission fits");
    assert_eq!(transaction.used_nodes(), Some(1));

    let error = transaction
        .try_enter_container(JsonContainerKind::Sequence, 2)
        .expect_err("nested container exceeds depth");
    assert_eq!(error.resource(), &JsonResource::Depth);
    assert_eq!(transaction.used_nodes(), Some(1));
}
