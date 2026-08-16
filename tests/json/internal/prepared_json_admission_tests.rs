// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Verifies prepared JSON admission limit checks through transactions.

use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;

/// Verifies point limits reject an event before cumulative accounting changes.
#[test]
fn test_prepared_admission_checks_point_limit_first() {
    let mut budget = JsonValueLimits::<JsonResource, usize>::builder()
        .max_nodes(2)
        .max_string_bytes(1)
        .max_payload_bytes(4)
        .build()
        .budget();
    let mut transaction = budget.transaction();
    let error = transaction
        .try_admit(JsonMeasurement::String { depth: 1, bytes: 2 })
        .expect_err("point string limit rejects the event");

    assert_eq!(error.resource(), &JsonResource::StringBytes);
    assert_eq!(transaction.used_nodes(), Some(0));
    assert_eq!(transaction.used_payload_bytes(), Some(0));
}
