// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for structured resource limit failures.

use qubit_budget::BudgetError;
use qubit_budget::InsufficientBudgetError;
use qubit_budget::LimitExceededError;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::Observation;
use qubit_budget::QuantityConversionError;
use qubit_budget::QuantityMeasurement;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    Depth,
}

#[test]
fn test_limit_exceeded_error_exposes_point_limit_facts() {
    let error = LimitExceededError {
        resource: TestResource::Depth,
        observed: Observation::Exact(4_usize),
        maximum: 3,
    };

    assert_eq!(error.resource(), &TestResource::Depth);
    assert_eq!(error.observation(), Observation::Exact(4));
    assert_eq!(error.exact_observed(), Some(4));
    assert_eq!(error.observed_lower_bound(), 4);
    assert_eq!(error.maximum(), 3);
    assert_eq!(error.into_resource(), TestResource::Depth);
}

#[test]
fn test_insufficient_budget_error_exposes_consumption_facts() {
    let error = InsufficientBudgetError {
        resource: TestResource::Depth,
        limit: 3_usize,
        remaining: 1,
        requested: 2,
    };

    assert_eq!(error.resource(), &TestResource::Depth);
    assert_eq!(error.limit(), 3);
    assert_eq!(error.remaining(), 1);
    assert_eq!(error.requested(), 2);
    assert_eq!(error.into_resource(), TestResource::Depth);
}

#[test]
fn test_measured_budget_error_wraps_insufficient_budget_failures() {
    let error = MeasuredBudgetError::from(InsufficientBudgetError {
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

#[test]
fn test_measured_budget_error_exposes_both_optional_sources() {
    let budget = MeasuredBudgetError::from(InsufficientBudgetError {
        resource: TestResource::Depth,
        limit: 3_usize,
        remaining: 1,
        requested: 2,
    });
    assert!(budget.budget_error().is_some());
    assert!(budget.quantity_error().is_none());

    let quantity = MeasuredBudgetError::<TestResource, usize>::quantity(
        TestResource::Depth,
        QuantityConversionError::new(QuantityMeasurement::Usize(256), "u8"),
    );
    assert!(quantity.budget_error().is_none());
    assert!(quantity.quantity_error().is_some());
}

#[test]
fn test_budget_error_aggregates_both_precise_budget_failures() {
    let point = BudgetError::from(LimitExceededError {
        resource: TestResource::Depth,
        observed: Observation::Exact(4_usize),
        maximum: 3,
    });
    let cumulative = BudgetError::from(InsufficientBudgetError {
        resource: TestResource::Depth,
        limit: 3_usize,
        remaining: 1,
        requested: 2,
    });

    assert_eq!(point.maximum(), Some(3));
    assert_eq!(cumulative.requested(), Some(2));
}
