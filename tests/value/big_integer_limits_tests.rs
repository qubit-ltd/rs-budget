use num_bigint::BigInt;
use qubit_budget::BigIntegerLimits;
use qubit_budget::BudgetError;
use qubit_budget::Observation;
use qubit_budget::ResourceLimit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    Bits,
    Digits,
}

#[test]
fn test_zero_has_no_significant_decimal_digits() {
    let limits = BigIntegerLimits::empty()
        .with_significant_decimal_digits_limit(ResourceLimit::new(TestResource::Digits, 0));
    limits
        .check(&BigInt::from(0))
        .expect("zero has no significant decimal digits");
}

#[test]
fn test_obvious_digit_overflow_reports_lower_bound() {
    let huge = BigInt::from(1_u8) << 1_000_000_u32;
    let limits = BigIntegerLimits::empty()
        .with_significant_decimal_digits_limit(ResourceLimit::new(TestResource::Digits, 16));
    assert!(matches!(
        limits.check(&huge),
        Err(BudgetError::LimitExceeded {
            resource: TestResource::Digits,
            observed: Observation::AtLeast(17),
            maximum: 16,
        })
    ));
}

#[test]
fn test_magnitude_bits_limit_is_checked_before_digits() {
    let value = BigInt::from(1_u8) << 10;
    let limits = BigIntegerLimits::empty()
        .with_magnitude_bits_limit(ResourceLimit::new(TestResource::Bits, 8))
        .with_significant_decimal_digits_limit(ResourceLimit::new(TestResource::Digits, 1));
    assert!(matches!(
        limits.check(&value),
        Err(BudgetError::LimitExceeded {
            resource: TestResource::Bits,
            observed: Observation::Exact(11),
            maximum: 8,
        })
    ));
}
