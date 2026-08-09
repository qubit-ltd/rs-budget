// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::ResourceBudget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    Bytes,
}

#[test]
fn test_budget_error_exposes_resource_limit_remaining_and_request() {
    let mut budget = ResourceBudget::new(TestResource::Bytes, 3_u64);
    budget.try_consume(2).expect("two units should fit");
    let error = budget
        .try_consume(2)
        .expect_err("two more units should exceed the remainder");
    assert_eq!(error.resource(), &TestResource::Bytes);
    assert_eq!(error.limit(), 3_u64);
    assert_eq!(error.remaining(), 1);
    assert_eq!(error.requested(), 2);
    assert_eq!(error.used(), 2);
    assert_eq!(error.checked_attempted(), Some(4));
    assert_eq!(error.into_resource(), TestResource::Bytes);
}

#[test]
fn test_budget_error_display_contains_the_failed_facts() {
    let mut budget = ResourceBudget::new(TestResource::Bytes, 1_u64);
    let error = budget
        .try_consume(2)
        .expect_err("two units should exceed one unit");
    let display = error.to_string();
    assert_eq!(
        display,
        "resource Bytes requested 2, but only 1 of 1 remains",
    );
}
