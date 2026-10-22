// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Verifies value transaction commit and rollback semantics.

use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonContainerKind;
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;

/// Verifies a transaction exposes whether any value limit is active.
#[test]
fn test_value_transaction_reports_whether_limits_are_configured() {
    let mut unlimited_budget = JsonValueLimits::<JsonResource, usize>::new().budget();
    assert!(!unlimited_budget.transaction().has_limits());

    let mut limited_budget = JsonValueLimits::<JsonResource, usize>::builder()
        .max_nodes(1)
        .build()
        .budget();
    assert!(limited_budget.transaction().has_limits());
}

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

    transaction
        .commit()
        .expect("successful transaction commits");

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
    let commit_error = transaction
        .commit()
        .expect_err("poisoned transaction cannot commit");
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

    transaction
        .check_container_count(JsonContainerKind::Map, 1)
        .expect("the first map entry fits");
    let error = transaction
        .check_container_count(JsonContainerKind::Map, 2)
        .expect_err("the second map entry exceeds the configured limit");

    assert_eq!(error.resource(), &JsonResource::MapEntries);
}

/// Verifies entering a container accounts for its node before its children.
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

/// Verifies successful admission previews use existing staged capacity without charging it.
#[test]
fn test_check_admit_previews_without_charging_and_try_admit_charges_once() {
    let mut budget = JsonValueLimits::<JsonResource, usize>::builder()
        .max_nodes(2)
        .max_payload_bytes(5)
        .build()
        .budget();
    let mut transaction = budget.transaction();
    transaction
        .try_admit(JsonMeasurement::Null { depth: 1 })
        .expect("existing staged node is counted");
    let measurement = JsonMeasurement::String { depth: 1, bytes: 3 };

    transaction.check_admit(measurement).expect("preview fits");
    transaction
        .check_admit(measurement)
        .expect("preview can repeat");
    assert_eq!(transaction.used_nodes(), Some(1));
    assert_eq!(transaction.used_payload_bytes(), Some(0));

    transaction
        .try_admit(measurement)
        .expect("actual admission fits");
    transaction.commit().expect("transaction commits");
    assert_eq!(budget.used_nodes(), Some(2));
    assert_eq!(budget.used_payload_bytes(), Some(3));
}

/// Verifies a failed preview poisons the transaction and preserves the target.
#[test]
fn test_failed_check_admit_preserves_first_error_and_prevents_commit() {
    let mut budget = JsonValueLimits::<JsonResource, usize>::builder()
        .max_nodes(1)
        .max_payload_bytes(2)
        .build()
        .budget();
    let mut transaction = budget.transaction();
    let measurement = JsonMeasurement::String { depth: 1, bytes: 3 };

    let first = transaction
        .check_admit(measurement)
        .expect_err("preview exceeds payload capacity");
    let repeated = transaction
        .try_admit(JsonMeasurement::Null { depth: 1 })
        .expect_err("preview failure poisons later admissions");
    assert_eq!(repeated.resource(), first.resource());
    let commit_error = transaction
        .commit()
        .expect_err("poisoned transaction cannot commit");
    assert_eq!(commit_error.resource(), first.resource());
    assert_eq!(budget.used_nodes(), Some(0));
    assert_eq!(budget.used_payload_bytes(), Some(0));
}

/// Verifies previews reuse point and cumulative checks for non-string measurements.
#[test]
fn test_check_admit_checks_point_node_and_key_limits() {
    let mut budget = JsonValueLimits::<JsonResource, usize>::builder()
        .max_depth(1)
        .max_nodes(1)
        .max_map_entries(1)
        .max_key_bytes(2)
        .build()
        .budget();
    let mut transaction = budget.transaction();

    let depth_error = transaction
        .check_admit(JsonMeasurement::Null { depth: 2 })
        .expect_err("point limits are checked before cumulative capacity");
    assert_eq!(depth_error.resource(), &JsonResource::Depth);
}

/// Verifies successful container and key previews leave unlimited staged usage unchanged.
#[test]
fn test_check_admit_previews_container_and_key_with_unlimited_budget() {
    let mut budget = JsonValueLimits::<JsonResource, usize>::new().budget();
    let mut transaction = budget.transaction();

    transaction
        .check_admit(JsonMeasurement::Object {
            depth: 1,
            entries: 1,
        })
        .expect("container preview fits");
    transaction
        .check_admit(JsonMeasurement::Key { bytes: 4 })
        .expect("key preview fits");
    assert_eq!(transaction.used_nodes(), None);
    assert_eq!(transaction.used_payload_bytes(), None);

    transaction
        .try_admit(JsonMeasurement::Object {
            depth: 1,
            entries: 1,
        })
        .expect("container admission fits");
    transaction
        .try_admit(JsonMeasurement::Key { bytes: 4 })
        .expect("key admission fits");
    transaction.commit().expect("transaction commits");
    assert_eq!(budget.used_nodes(), None);
}

/// Verifies preview conversion errors retain caller-defined resource identity.
#[test]
fn test_check_admit_preserves_custom_resource_on_quantity_error() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Resource {
        Text,
    }

    let mut budget = JsonValueLimits::<Resource, u8>::builder()
        .string_bytes_limit(ResourceLimit::new(Resource::Text, u8::MAX))
        .build()
        .budget();
    let mut transaction = budget.transaction();
    let error = transaction
        .check_admit(JsonMeasurement::String {
            depth: 1,
            bytes: 256,
        })
        .expect_err("the byte count cannot be converted to u8");

    assert_eq!(error.resource(), &Resource::Text);
}

/// Verifies poisoning preserves the first failure and publishes no staged
/// usage.
#[test]
fn test_poison_rejects_later_admission_and_commit_without_publishing() {
    let mut budget = JsonValueLimits::<JsonResource, u8>::builder()
        .max_nodes(3)
        .max_string_bytes(2)
        .max_payload_bytes(3)
        .build()
        .budget();
    let mut transaction = budget.transaction();
    transaction
        .try_admit(JsonMeasurement::String { depth: 1, bytes: 2 })
        .expect("first string fits");
    assert_eq!(
        (transaction.used_nodes(), transaction.used_payload_bytes()),
        (Some(1), Some(2))
    );

    let first = transaction
        .try_admit(JsonMeasurement::String {
            depth: 1,
            bytes: 256,
        })
        .expect_err("u8 cannot represent the configured string measurement");
    let second = transaction
        .try_admit(JsonMeasurement::Null { depth: 1 })
        .expect_err("poisoned transaction retains its first failure");
    assert_eq!(first.resource(), second.resource());
    assert_eq!(
        first
            .quantity_error()
            .expect("quantity failure")
            .measurement(),
        second
            .quantity_error()
            .expect("same quantity failure")
            .measurement(),
    );
    let final_error = transaction
        .commit()
        .expect_err("poisoned transaction cannot commit");
    assert_eq!(
        first
            .quantity_error()
            .expect("quantity failure")
            .measurement(),
        final_error
            .quantity_error()
            .expect("same quantity failure")
            .measurement(),
    );
    assert_eq!(
        (budget.used_nodes(), budget.used_payload_bytes()),
        (Some(0), Some(0))
    );

    let mut next = budget.transaction();
    next.try_admit(JsonMeasurement::Null { depth: 1 })
        .expect("a new transaction can use the unchanged budget");
    next.commit().expect("new transaction commits");
    assert_eq!(budget.used_nodes(), Some(1));
}
