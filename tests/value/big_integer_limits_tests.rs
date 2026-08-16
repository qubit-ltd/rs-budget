// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use num_bigint::BigInt;
use proptest::prelude::any;
use proptest::prelude::prop_assert;
use proptest::prelude::proptest;
use qubit_budget::BigIntegerLimits;
use qubit_budget::BudgetError;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::Observation;
use qubit_budget::ResourceLimit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    Bits,
    Digits,
}

#[test]
fn test_zero_has_no_significant_decimal_digits() {
    let limits = BigIntegerLimits::new()
        .with_significant_decimal_digits_limit(ResourceLimit::new(TestResource::Digits, 0_u64));
    limits
        .check(&BigInt::from(0))
        .expect("zero has no significant decimal digits");
}

#[test]
fn test_obvious_digit_overflow_reports_lower_bound() {
    let huge = BigInt::from(1_u8) << 1_000_000_u32;
    let limits = BigIntegerLimits::new()
        .with_significant_decimal_digits_limit(ResourceLimit::new(TestResource::Digits, 16_u64));
    assert!(matches!(
        limits.check(&huge),
        Err(MeasuredBudgetError::Budget(BudgetError::LimitExceeded {
            resource: TestResource::Digits,
            observed: Observation::AtLeast(17),
            maximum: 16,
        }))
    ));
}

#[test]
fn test_magnitude_bits_limit_is_checked_before_digits() {
    let value = BigInt::from(1_u8) << 10;
    let limits = BigIntegerLimits::new()
        .with_magnitude_bits_limit(ResourceLimit::new(TestResource::Bits, 8_u64))
        .with_significant_decimal_digits_limit(ResourceLimit::new(TestResource::Digits, 1_u64));
    assert!(matches!(
        limits.check(&value),
        Err(MeasuredBudgetError::Budget(BudgetError::LimitExceeded {
            resource: TestResource::Bits,
            observed: Observation::Exact(11),
            maximum: 8,
        }))
    ));
}

#[test]
fn test_big_integer_limits_support_usize_quantities() {
    let limits = BigIntegerLimits::<TestResource, usize>::new()
        .with_magnitude_bits_limit(ResourceLimit::new(TestResource::Bits, 8));
    let error = limits
        .check(&(BigInt::from(1_u8) << 10))
        .expect_err("eleven bits exceed eight");

    assert!(matches!(
        error,
        MeasuredBudgetError::Budget(BudgetError::LimitExceeded {
            resource: TestResource::Bits,
            observed: Observation::Exact(11),
            maximum: 8,
        })
    ));
}

#[test]
fn test_big_integer_accessors_and_unconfigured_limits() {
    let limits = BigIntegerLimits::new()
        .with_magnitude_bits_limit(ResourceLimit::new(TestResource::Bits, 8_u64))
        .with_significant_decimal_digits_limit(ResourceLimit::new(TestResource::Digits, 3_u64));
    assert_eq!(limits.magnitude_bits_limit().unwrap().maximum(), 8);
    assert_eq!(
        limits.significant_decimal_digits_limit().unwrap().maximum(),
        3
    );
    BigIntegerLimits::<TestResource>::new()
        .check(&BigInt::from(123456))
        .expect("unconfigured limits accept every value");
}

#[test]
fn test_big_integer_default_is_unconfigured() {
    BigIntegerLimits::<TestResource>::default()
        .check(&BigInt::from(1))
        .expect("default limits accept every value");
}

proptest! {
    #[test]
    fn test_significant_decimal_digit_limit_matches_exact_measurement(
        sign in any::<bool>(),
        shift in 0_u32..=1024,
        maximum in 0_u64..=400,
    ) {
        let mut value = BigInt::from(1_u8) << shift;
        if sign {
            value = -value;
        }
        let text = value.to_str_radix(10);
        let digits = text.strip_prefix('-').unwrap_or(&text).len() as u64;
        let limits = BigIntegerLimits::new()
            .with_significant_decimal_digits_limit(ResourceLimit::new(
                TestResource::Digits,
                maximum,
            ));

        let result = limits.check(&value);
        if digits <= maximum {
            prop_assert!(result.is_ok());
        } else {
            let error = result.expect_err("values over the digit limit must fail");
            prop_assert!(error.budget_error().is_some());
            prop_assert!(
                error
                    .budget_error()
                    .and_then(BudgetError::observed_lower_bound)
                    .is_some_and(|observed| observed > maximum)
            );
        }
    }
}
