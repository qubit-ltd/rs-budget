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
use qubit_budget::QuantityMeasurement;
use qubit_budget::ResourceLimit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    Bits,
    Digits,
}

#[test]
fn test_zero_has_no_significant_decimal_digits() {
    let limits = BigIntegerLimits::builder()
        .significant_decimal_digits_limit(ResourceLimit::new(TestResource::Digits, 0_u64))
        .build();
    limits
        .check(&BigInt::from(0))
        .expect("zero has no significant decimal digits");
}

#[test]
fn test_obvious_digit_overflow_reports_lower_bound() {
    let huge = BigInt::from(1_u8) << 1_000_000_u32;
    let limits = BigIntegerLimits::builder()
        .significant_decimal_digits_limit(ResourceLimit::new(TestResource::Digits, 16_u64))
        .build();
    assert!(matches!(
        limits.check(&huge),
        Err(MeasuredBudgetError::Budget(BudgetError::LimitExceeded {
            resource: TestResource::Digits,
            observed: Observation::AtLeast(250_001),
            maximum: 16,
        }))
    ));
}

#[test]
fn test_magnitude_bits_limit_is_checked_before_digits() {
    let value = BigInt::from(1_u8) << 10;
    let limits = BigIntegerLimits::builder()
        .magnitude_bits_limit(ResourceLimit::new(TestResource::Bits, 8_u64))
        .significant_decimal_digits_limit(ResourceLimit::new(TestResource::Digits, 1_u64))
        .build();
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
    let limits = BigIntegerLimits::<TestResource, usize>::builder()
        .magnitude_bits_limit(ResourceLimit::new(TestResource::Bits, 8))
        .build();
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
    let limits = BigIntegerLimits::builder()
        .magnitude_bits_limit(ResourceLimit::new(TestResource::Bits, 8_u64))
        .significant_decimal_digits_limit(ResourceLimit::new(TestResource::Digits, 3_u64))
        .build();
    assert_eq!(limits.magnitude_bits_limit().unwrap().maximum(), 8);
    assert_eq!(limits.significant_decimal_digits_limit().unwrap().maximum(), 3);
    BigIntegerLimits::<TestResource>::builder()
        .build()
        .check(&BigInt::from(123456))
        .expect("unconfigured limits accept every value");
}

#[test]
fn test_big_integer_default_is_unconfigured() {
    BigIntegerLimits::<TestResource>::default()
        .check(&BigInt::from(1))
        .expect("default limits accept every value");
}

#[test]
fn test_big_integer_check_accepts_values_at_both_limits() {
    let limits = BigIntegerLimits::builder()
        .magnitude_bits_limit(ResourceLimit::new(TestResource::Bits, 8_u64))
        .significant_decimal_digits_limit(ResourceLimit::new(TestResource::Digits, 3_u64))
        .build();
    limits
        .check(&BigInt::from(255))
        .expect("the value should fit both configured limits");
    assert_eq!(limits.into_builder().build(), limits);
}

#[test]
fn test_decimal_digits_ignore_unconfigured_bit_conversion() {
    let value = BigInt::from(1_u8) << 256_usize;
    for maximum in [78_u8, 79, 100] {
        let limits = BigIntegerLimits::builder()
            .significant_decimal_digits_limit(ResourceLimit::new("digits", maximum))
            .build();
        limits
            .check(&value)
            .expect("78 decimal digits fit the configured limit");
    }
    let limits = BigIntegerLimits::builder()
        .significant_decimal_digits_limit(ResourceLimit::new("digits", 77_u8))
        .build();
    assert!(
        limits
            .check(&value)
            .expect_err("78 exceeds 77")
            .budget_error()
            .is_some()
    );
}

#[test]
fn test_unrepresentable_decimal_lower_bound_is_not_reported_as_exact() {
    let value = BigInt::from(1_u8) << 16_384_usize;
    let limits = BigIntegerLimits::builder()
        .significant_decimal_digits_limit(ResourceLimit::new("digits", 255_u8))
        .build();
    let error = limits
        .check(&value)
        .expect_err("the proven digit lower bound exceeds u8");
    let source = error.quantity_error().expect("lower bound cannot fit u8");
    assert_eq!(source.measurement(), QuantityMeasurement::AtLeastU64(4097));
    assert_eq!(source.target(), "u8");
}

#[test]
fn test_u128_maximum_digit_limit_does_not_overflow_estimate() {
    let value = BigInt::from(1_u8) << 256_usize;
    let limits = BigIntegerLimits::<TestResource, u128>::builder()
        .significant_decimal_digits_limit(ResourceLimit::new(TestResource::Digits, u128::MAX))
        .build();
    limits.check(&value).expect("small integer fits the maximum u128 limit");
}

#[test]
fn test_narrow_magnitude_limit_keeps_bit_error_precedence() {
    let value = BigInt::from(1_u8) << 256_usize;
    let limits = BigIntegerLimits::builder()
        .magnitude_bits_limit(ResourceLimit::new("bits", 255_u8))
        .significant_decimal_digits_limit(ResourceLimit::new("digits", 100_u8))
        .build();
    let error = limits.check(&value).expect_err("257 magnitude bits do not fit u8");
    let source = error.quantity_error().expect("bit measurement cannot fit u8");
    assert_eq!(source.measurement(), QuantityMeasurement::U64(257));
    assert_eq!(source.target(), "u8");
}

#[test]
fn test_decimal_digit_contract_across_supported_quantities() {
    fn assert_digit_contract<Q: qubit_budget::ResourceQuantity>(shift: usize, maximum: u8) {
        let positive = BigInt::from(1_u8) << shift;
        for value in [positive.clone(), -positive] {
            let digits = value.to_str_radix(10).trim_start_matches('-').len();
            let limits = BigIntegerLimits::<_, Q>::builder()
                .significant_decimal_digits_limit(ResourceLimit::new(
                    TestResource::Digits,
                    Q::try_from_u64(u64::from(maximum)).expect("maximum fits quantity"),
                ))
                .build();
            assert_eq!(limits.check(&value).is_ok(), digits <= usize::from(maximum));
        }
    }

    macro_rules! assert_quantity_contract {
        ($quantity:ty) => {
            for shift in [0, 1, 256, 1_024, 2_048] {
                for maximum in [0, 1, 77, 78, 255] {
                    assert_digit_contract::<$quantity>(shift, maximum);
                }
            }
        };
    }
    assert_quantity_contract!(u8);
    assert_quantity_contract!(u16);
    assert_quantity_contract!(u32);
    assert_quantity_contract!(u64);
    assert_quantity_contract!(u128);
    assert_quantity_contract!(usize);
}

#[test]
fn test_decimal_digit_boundaries_around_powers_of_ten() {
    for exponent in 1_u32..=100 {
        let power = BigInt::from(10_u8).pow(exponent);
        for (value, digits) in [
            (&power - 1, exponent as usize),
            (power.clone(), exponent as usize + 1),
            (&power + 1, exponent as usize + 1),
        ] {
            let limits = BigIntegerLimits::builder()
                .significant_decimal_digits_limit(ResourceLimit::new(TestResource::Digits, digits as u64))
                .build();
            limits.check(&value).expect("value fits its exact decimal digit count");
        }
    }
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
        let limits = BigIntegerLimits::builder()
            .significant_decimal_digits_limit(ResourceLimit::new(
                TestResource::Digits,
                maximum,
            )).build();

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
