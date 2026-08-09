// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::time::Duration;

use qubit_budget::DurationBudget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    OperationDuration,
}

#[test]
fn test_duration_error_exposes_all_structured_facts() {
    let mut budget = DurationBudget::new(
        TestResource::OperationDuration,
        Duration::from_secs(1),
    );
    let error = budget
        .try_consume(Duration::from_secs(2))
        .expect_err("two seconds should exceed one second");
    assert_eq!(error.resource(), &TestResource::OperationDuration);
    assert_eq!(error.limit(), Duration::from_secs(1));
    assert_eq!(error.remaining(), Duration::from_secs(1));
    assert_eq!(error.requested(), Duration::from_secs(2));
    assert_eq!(error.into_resource(), TestResource::OperationDuration);
}

#[test]
fn test_duration_error_can_be_displayed() {
    let mut budget = DurationBudget::new(
        TestResource::OperationDuration,
        Duration::from_secs(1),
    );
    let error = budget
        .try_consume(Duration::from_secs(2))
        .expect_err("two seconds should exceed one second");
    assert!(error.to_string().contains("requested"));
}
