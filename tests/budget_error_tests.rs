// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for structured resource limit failures.

use qubit_budget::BudgetError;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::Observation;
use qubit_budget::QuantityConversionError;
use qubit_budget::QuantityMeasurement;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    Depth,
}

#[test]
fn test_limit_exceeded_error_exposes_only_point_limit_facts() {
    let error = BudgetError::LimitExceeded {
        resource: TestResource::Depth,
        observed: Observation::Exact(4_usize),
        maximum: 3,
    };

    assert_eq!(error.resource(), &TestResource::Depth);
    assert_eq!(error.limit(), None);
    assert_eq!(error.observation(), Some(Observation::Exact(4)));
    assert_eq!(error.exact_observed(), Some(4));
    assert_eq!(error.observed_lower_bound(), Some(4));
    assert_eq!(error.maximum(), Some(3));
    assert_eq!(error.remaining(), None);
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
    assert_eq!(error.observation(), None);
    assert_eq!(error.exact_observed(), None);
    assert_eq!(error.observed_lower_bound(), None);
    assert_eq!(error.maximum(), None);
    assert_eq!(error.remaining(), Some(1));
    assert_eq!(error.requested(), Some(2));
    assert_eq!(error.into_resource(), TestResource::Depth);
}

#[test]
fn test_measured_budget_error_exposes_resource_for_budget_failures() {
    let error = MeasuredBudgetError::Budget(BudgetError::Insufficient {
        resource: TestResource::Depth,
        limit: 3_usize,
        remaining: 1,
        requested: 2,
    });

    assert_eq!(error.resource(), &TestResource::Depth);
    assert_eq!(error.into_resource(), TestResource::Depth);
}

#[test]
fn test_measured_budget_error_exposes_resource_for_quantity_failures() {
    let error = MeasuredBudgetError::<TestResource, usize>::quantity(
        TestResource::Depth,
        QuantityConversionError::new(QuantityMeasurement::Usize(256), "u8"),
    );

    assert_eq!(error.resource(), &TestResource::Depth);
    assert_eq!(error.into_resource(), TestResource::Depth);
}
