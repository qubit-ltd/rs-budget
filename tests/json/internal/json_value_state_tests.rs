// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Verifies cumulative JSON value state through its public transaction API.

use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;

/// Verifies node and payload state are staged together before commit.
#[test]
fn test_state_stages_node_and_payload_usage() {
    let mut budget = JsonValueLimits::<JsonResource, usize>::new()
        .with_max_nodes(2)
        .with_max_payload_bytes(4)
        .budget();
    let mut transaction = budget.transaction();
    transaction
        .try_admit(JsonMeasurement::String { depth: 1, bytes: 3 })
        .expect("string fits");

    assert_eq!(transaction.used_nodes(), Some(1));
    assert_eq!(transaction.used_payload_bytes(), Some(3));
    assert_eq!(budget.used_nodes(), Some(0));
    assert_eq!(budget.used_payload_bytes(), Some(0));
}
