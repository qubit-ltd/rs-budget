// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for structured resource limit failures.

use qubit_budget::BudgetError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    Depth,
}

#[test]
fn test_limit_exceeded_error_exposes_only_point_limit_facts() {
    let error = BudgetError::LimitExceeded {
        resource: TestResource::Depth,
        actual: 4_usize,
        maximum: 3,
    };

    assert_eq!(error.resource(), &TestResource::Depth);
    assert_eq!(error.limit(), None);
    assert_eq!(error.actual(), Some(4));
    assert_eq!(error.maximum(), Some(3));
    assert_eq!(error.remaining(), None);
    assert_eq!(error.in_use(), None);
    assert_eq!(error.requested(), None);
    assert_eq!(error.into_resource(), TestResource::Depth);
}

#[test]
fn test_insufficient_error_exposes_only_consumption_facts() {
    let error = BudgetError::Insufficient {
        resource: TestResource::Depth,
        limit: 3_usize,
        remaining: 1,
        requested: 2,
    };

    assert_eq!(error.resource(), &TestResource::Depth);
    assert_eq!(error.limit(), Some(3));
    assert_eq!(error.actual(), None);
    assert_eq!(error.maximum(), None);
    assert_eq!(error.remaining(), Some(1));
    assert_eq!(error.in_use(), None);
    assert_eq!(error.requested(), Some(2));
    assert_eq!(error.into_resource(), TestResource::Depth);
}

#[test]
fn test_invalid_release_error_exposes_only_release_facts() {
    let error = BudgetError::InvalidRelease {
        resource: TestResource::Depth,
        limit: 3_usize,
        in_use: 1,
        requested: 2,
    };

    assert_eq!(error.resource(), &TestResource::Depth);
    assert_eq!(error.limit(), Some(3));
    assert_eq!(error.actual(), None);
    assert_eq!(error.maximum(), None);
    assert_eq!(error.remaining(), None);
    assert_eq!(error.in_use(), Some(1));
    assert_eq!(error.requested(), Some(2));
    assert_eq!(error.into_resource(), TestResource::Depth);
}
