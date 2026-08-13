// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for resource-bound point limits.

use std::time::Duration;

use qubit_budget::BudgetError;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::Observation;
use qubit_budget::ResourceLimit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    Depth,
}

#[test]
fn test_resource_limit_is_clone_and_copy_when_its_fields_are() {
    fn assert_clone_and_copy<T: Clone + Copy>() {}

    assert_clone_and_copy::<ResourceLimit<TestResource, usize>>();
}

#[test]
fn test_resource_limit_accepts_exact_and_reports_over_limit_facts() {
    let limit = ResourceLimit::new(TestResource::Depth, 3_usize);

    assert_eq!(limit.resource(), &TestResource::Depth);
    assert_eq!(limit.maximum(), 3);
    limit.check(3).expect("the exact depth should fit");
    assert!(matches!(
        limit.check(4),
        Err(BudgetError::LimitExceeded {
            resource: TestResource::Depth,
            observed: Observation::Exact(4),
            maximum: 3,
        })
    ));
}

#[test]
fn test_resource_limit_supports_duration_measurements() {
    let maximum = Duration::from_secs(3);
    let actual = Duration::from_secs(4);
    let limit = ResourceLimit::new(TestResource::Depth, maximum);

    assert!(matches!(
        limit.check(actual),
        Err(BudgetError::<TestResource, Duration>::LimitExceeded {
            resource: TestResource::Depth,
            observed: Observation::Exact(reported_actual),
            maximum: reported_maximum,
        }) if reported_actual == actual && reported_maximum == maximum
    ));
}

#[test]
fn test_resource_limit_converts_usize_and_u64_measurements() {
    let limit = ResourceLimit::new(TestResource::Depth, 3_u8);

    limit
        .check_usize(3)
        .expect("the exact usize measurement should fit");
    assert!(matches!(
        limit.check_u64(4),
        Err(MeasuredBudgetError::Budget(BudgetError::LimitExceeded {
            resource: TestResource::Depth,
            observed: Observation::Exact(4),
            maximum: 3,
        }))
    ));
    assert!(matches!(
        limit.check_usize(usize::from(u8::MAX) + 1),
        Err(MeasuredBudgetError::Quantity { .. })
    ));
}
