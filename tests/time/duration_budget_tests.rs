// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::time::Duration;

use qubit_budget::BudgetError;
use qubit_budget::DurationBudget;
use qubit_budget::ResourceLimit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    OperationDuration,
}

#[test]
fn test_try_consume_uses_remaining_duration_and_is_atomic_on_failure() {
    let limit = Duration::from_secs(5);
    let mut budget = DurationBudget::new(TestResource::OperationDuration, limit);
    budget
        .try_consume(Duration::from_secs(2))
        .expect("two seconds should fit");
    assert!(matches!(
        budget.try_consume(Duration::from_secs(4)),
        Err(BudgetError::<_, Duration>::Insufficient {
            resource: TestResource::OperationDuration,
            limit,
            remaining,
            requested,
        }) if limit == Duration::from_secs(5)
            && remaining == Duration::from_secs(3)
            && requested == Duration::from_secs(4)
    ));
    assert_eq!(budget.remaining(), Duration::from_secs(3));
    assert_eq!(budget.used(), Duration::from_secs(2));
}

#[test]
fn test_consume_available_returns_the_exact_consumed_duration() {
    let mut budget = DurationBudget::new(TestResource::OperationDuration, Duration::from_secs(3));
    assert_eq!(
        budget.consume_available(Duration::from_secs(5)),
        Duration::from_secs(3)
    );
    assert_eq!(budget.remaining(), Duration::ZERO);
}

#[test]
fn test_duration_budget_accessors_preserve_resource_and_limit() {
    let limit = Duration::from_secs(3);
    let budget = DurationBudget::new(TestResource::OperationDuration, limit);
    assert_eq!(budget.resource(), &TestResource::OperationDuration);
    assert_eq!(budget.limit(), limit);
    assert_eq!(
        budget.resource_limit().resource(),
        &TestResource::OperationDuration
    );
    assert_eq!(budget.resource_limit().maximum(), limit);
    assert_eq!(budget.remaining(), limit);
    assert_eq!(budget.used(), Duration::ZERO);
}

#[test]
fn test_from_limit_preserves_the_resource_limit() {
    let limit = ResourceLimit::new(TestResource::OperationDuration, Duration::from_secs(3));
    let budget = DurationBudget::from_limit(limit);
    assert_eq!(budget.resource_limit(), &limit);
}
