use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use qubit_budget::BigDecimalLimits;
use qubit_budget::BigIntegerLimits;
use qubit_budget::ResourceLimit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    Digits,
    Scale,
}

#[test]
fn test_minimum_scale_reports_exact_unsigned_magnitude() {
    let value = BigDecimal::new(BigInt::from(1), i64::MIN);
    let limits = BigDecimalLimits::empty().with_scale_magnitude_limit(
        ResourceLimit::new(TestResource::Scale, 150_000),
    );
    let error = limits.check(&value).expect_err("minimum scale must fail");
    assert_eq!(error.exact_observed(), Some(i64::MIN.unsigned_abs()));
}

#[test]
fn test_scale_is_checked_before_coefficient() {
    let value = BigDecimal::new(BigInt::from(10).pow(1000), 200_000);
    let limits = BigDecimalLimits::empty()
        .with_scale_magnitude_limit(ResourceLimit::new(
            TestResource::Scale,
            150_000,
        ))
        .with_coefficient_limits(
            BigIntegerLimits::empty().with_significant_decimal_digits_limit(
                ResourceLimit::new(TestResource::Digits, 1),
            ),
        );
    assert_eq!(
        limits.check(&value).unwrap_err().resource(),
        &TestResource::Scale
    );
}
