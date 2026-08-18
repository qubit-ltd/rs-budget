// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow explicit-imports
use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use qubit_budget::BigDecimalLimits;
use qubit_budget::BigIntegerLimits;
use qubit_budget::BudgetError;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::ResourceLimit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    Digits,
    Scale,
}

#[test]
fn test_minimum_scale_reports_exact_unsigned_magnitude() {
    let value = BigDecimal::new(BigInt::from(1), i64::MIN);
    let limits = BigDecimalLimits::builder()
        .scale_magnitude_limit(ResourceLimit::new(TestResource::Scale, 150_000_u64))
        .build();
    let error = limits.check(&value).expect_err("minimum scale must fail");
    assert_eq!(
        error
            .budget_error()
            .and_then(|budget_error| budget_error.exact_observed()),
        Some(i64::MIN.unsigned_abs())
    );
}

#[test]
fn test_scale_is_checked_before_coefficient() {
    let value = BigDecimal::new(BigInt::from(10).pow(1000), 200_000);
    let limits = BigDecimalLimits::builder()
        .scale_magnitude_limit(ResourceLimit::new(TestResource::Scale, 150_000_u64))
        .coefficient_limits(
            BigIntegerLimits::builder()
                .significant_decimal_digits_limit(ResourceLimit::new(TestResource::Digits, 1_u64))
                .build(),
        )
        .build();
    let error = limits.check(&value).expect_err("scale must be checked first");
    assert_eq!(
        error.budget_error().map(BudgetError::resource),
        Some(&TestResource::Scale)
    );
}

#[test]
fn test_big_decimal_limits_support_usize_quantities() {
    let value = BigDecimal::new(BigInt::from(1), 9);
    let limits = BigDecimalLimits::<TestResource, usize>::builder()
        .scale_magnitude_limit(ResourceLimit::new(TestResource::Scale, 8))
        .build();
    let error = limits.check(&value).expect_err("scale must exceed the limit");

    assert!(matches!(
        error,
        MeasuredBudgetError::Budget(BudgetError::LimitExceeded {
            resource: TestResource::Scale,
            ..
        })
    ));
}

#[test]
fn test_big_decimal_accessors_and_unconfigured_limits() {
    let coefficient = BigIntegerLimits::builder()
        .magnitude_bits_limit(ResourceLimit::new(TestResource::Digits, 8_u64))
        .build();
    let limits = BigDecimalLimits::builder()
        .coefficient_limits(coefficient)
        .scale_magnitude_limit(ResourceLimit::new(TestResource::Scale, 3))
        .build();
    assert_eq!(limits.coefficient_limits().magnitude_bits_limit().unwrap().maximum(), 8);
    assert_eq!(limits.scale_magnitude_limit().unwrap().maximum(), 3);
    BigDecimalLimits::<TestResource>::builder()
        .build()
        .check(&BigDecimal::from(1))
        .expect("unconfigured limits accept every value");
}

#[test]
fn test_big_decimal_default_is_unconfigured() {
    let limits = BigDecimalLimits::<TestResource>::default();
    limits
        .check(&BigDecimal::from(1))
        .expect("default limits accept every value");
}

#[test]
fn test_big_decimal_u64_coefficient_limit_is_checked() {
    let limits = BigDecimalLimits::<TestResource, u64>::builder()
        .coefficient_limits(
            BigIntegerLimits::builder()
                .significant_decimal_digits_limit(ResourceLimit::new(TestResource::Digits, 1_u64))
                .build(),
        )
        .build();
    let error = limits
        .check(&BigDecimal::from(100))
        .expect_err("coefficient digits should exceed the limit");
    assert_eq!(
        error.budget_error().map(BudgetError::resource),
        Some(&TestResource::Digits)
    );
}
