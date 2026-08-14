// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests committed JSON-value budget accounting and transaction lifetimes.

use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;

use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;

/// Verifies a rejected payload event does not change the transaction snapshot.
#[test]
fn test_payload_rejection_preserves_working_nodes_and_payload() {
    let mut budget = JsonValueLimits::empty()
        .with_max_nodes(1)
        .with_max_payload_bytes(2)
        .budget();
    let mut transaction = budget.transaction();

    assert!(
        transaction
            .try_admit(JsonMeasurement::String { depth: 1, bytes: 3 })
            .is_err()
    );
    assert_eq!(transaction.used_nodes(), Some(0));
    assert_eq!(transaction.used_payload_bytes(), Some(0));
}

/// Verifies dropping a failed transaction rolls back the complete value.
#[test]
fn test_drop_after_error_rolls_back_complete_value() {
    let mut budget = JsonValueLimits::empty().with_max_nodes(1).budget();
    {
        let mut transaction = budget.transaction();
        transaction
            .try_admit(JsonMeasurement::Null { depth: 1 })
            .expect("first event fits");
        assert!(
            transaction
                .try_admit(JsonMeasurement::Null { depth: 1 })
                .is_err()
        );
    }
    assert_eq!(budget.used_nodes(), Some(0));
}

/// Verifies a successful transaction changes committed state only on commit.
#[test]
fn test_commit_publishes_nodes_and_payload() {
    let mut budget = JsonValueLimits::empty()
        .with_max_nodes(2)
        .with_max_payload_bytes(4)
        .budget();
    let mut transaction = budget.transaction();
    transaction
        .try_admit(JsonMeasurement::Key { bytes: 1 })
        .expect("key fits");
    transaction
        .try_admit(JsonMeasurement::String { depth: 1, bytes: 3 })
        .expect("string fits");

    assert_eq!(transaction.used_nodes(), Some(1));
    assert_eq!(transaction.remaining_nodes(), Some(1));
    assert_eq!(transaction.used_payload_bytes(), Some(4));
    assert_eq!(transaction.remaining_payload_bytes(), Some(0));
    transaction.commit();

    assert_eq!(budget.used_nodes(), Some(1));
    assert_eq!(budget.remaining_nodes(), Some(1));
    assert_eq!(budget.used_payload_bytes(), Some(4));
    assert_eq!(budget.remaining_payload_bytes(), Some(0));
}

/// Verifies an uncommitted successful transaction rolls back on ordinary drop.
#[test]
fn test_drop_rolls_back_successful_admissions() {
    let mut budget = JsonValueLimits::empty()
        .with_max_nodes(1)
        .with_max_payload_bytes(1)
        .budget();
    {
        let mut transaction = budget.transaction();
        transaction
            .try_admit(JsonMeasurement::String { depth: 1, bytes: 1 })
            .expect("string fits");
    }

    assert_eq!(budget.used_nodes(), Some(0));
    assert_eq!(budget.used_payload_bytes(), Some(0));
}

/// Verifies unwinding drops a transaction without publishing its working state.
#[test]
fn test_panic_drop_rolls_back_working_state() {
    let mut budget = JsonValueLimits::empty()
        .with_max_nodes(1)
        .with_max_payload_bytes(1)
        .budget();

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut transaction = budget.transaction();
        transaction
            .try_admit(JsonMeasurement::String { depth: 1, bytes: 1 })
            .expect("string fits");
        panic!("force transaction drop during unwinding");
    }));

    assert!(result.is_err());
    assert_eq!(budget.used_nodes(), Some(0));
    assert_eq!(budget.used_payload_bytes(), Some(0));
}

/// Verifies a failed event leaves a transaction able to admit later events.
#[test]
fn test_try_admit_failure_keeps_transaction_usable() {
    let mut budget = JsonValueLimits::empty()
        .with_max_nodes(2)
        .with_max_payload_bytes(2)
        .budget();
    let mut transaction = budget.transaction();
    transaction
        .try_admit(JsonMeasurement::Null { depth: 1 })
        .expect("first node fits");

    assert!(
        transaction
            .try_admit(JsonMeasurement::String { depth: 1, bytes: 3 })
            .is_err()
    );
    assert_eq!(transaction.used_nodes(), Some(1));
    assert_eq!(transaction.used_payload_bytes(), Some(0));

    transaction
        .try_admit(JsonMeasurement::Null { depth: 1 })
        .expect("later node fits after rejection");
    transaction.commit();
    assert_eq!(budget.used_nodes(), Some(2));
    assert_eq!(budget.used_payload_bytes(), Some(0));
}

/// Verifies object keys consume only payload capacity, not value-node capacity.
#[test]
fn test_try_admit_key_consumes_only_payload() {
    let mut budget = JsonValueLimits::empty()
        .with_max_nodes(1)
        .with_max_payload_bytes(2)
        .budget();
    let mut transaction = budget.transaction();
    transaction
        .try_admit(JsonMeasurement::Key { bytes: 2 })
        .expect("key fits");
    transaction.commit();

    assert_eq!(budget.used_nodes(), Some(0));
    assert_eq!(budget.used_payload_bytes(), Some(2));
}

/// Verifies array and object events each consume one committed value node.
#[test]
fn test_try_admit_array_and_object_consume_nodes() {
    let mut budget = JsonValueLimits::empty().with_max_nodes(2).budget();
    let mut transaction = budget.transaction();
    transaction
        .try_admit(JsonMeasurement::Array { depth: 1, items: 0 })
        .expect("array fits");
    transaction
        .try_admit(JsonMeasurement::Object {
            depth: 1,
            entries: 0,
        })
        .expect("object fits");
    transaction.commit();

    assert_eq!(budget.used_nodes(), Some(2));
}

/// Verifies admission reports point, node, then cumulative failures by their
/// configured resource identity.
#[test]
fn test_try_admit_reports_deterministic_resource_priority() {
    let mut node_limited = JsonValueLimits::empty()
        .with_max_nodes(0)
        .with_max_string_bytes(0)
        .with_max_payload_bytes(0)
        .budget();
    let mut transaction = node_limited.transaction();
    let error = transaction
        .try_admit(JsonMeasurement::String { depth: 1, bytes: 1 })
        .expect_err("point limit rejects before cumulative limits");
    assert_eq!(error.resource(), &JsonResource::StringBytes);

    let mut point_limited = JsonValueLimits::empty()
        .with_max_nodes(0)
        .with_max_string_bytes(1)
        .with_max_payload_bytes(0)
        .budget();
    let mut transaction = point_limited.transaction();
    let error = transaction
        .try_admit(JsonMeasurement::String { depth: 1, bytes: 1 })
        .expect_err("node limit rejects after point checks");
    assert_eq!(error.resource(), &JsonResource::Nodes);

    let mut payload_limited = JsonValueLimits::empty()
        .with_max_nodes(1)
        .with_max_payload_bytes(0)
        .budget();
    let mut transaction = payload_limited.transaction();
    let error = transaction
        .try_admit(JsonMeasurement::String { depth: 1, bytes: 1 })
        .expect_err("payload limit rejects after node and point checks");
    assert_eq!(error.resource(), &JsonResource::PayloadBytes);
}

/// Verifies reset discards prior committed usage while retaining configuration.
#[test]
fn test_reset_clears_committed_state() {
    let mut budget = JsonValueLimits::empty()
        .with_max_nodes(1)
        .with_max_payload_bytes(1)
        .budget();
    let mut transaction = budget.transaction();
    transaction
        .try_admit(JsonMeasurement::String { depth: 1, bytes: 1 })
        .expect("string fits");
    transaction.commit();

    budget.reset();

    assert_eq!(budget.used_nodes(), Some(0));
    assert_eq!(budget.remaining_nodes(), Some(1));
    assert_eq!(budget.used_payload_bytes(), Some(0));
    assert_eq!(budget.remaining_payload_bytes(), Some(1));
}
