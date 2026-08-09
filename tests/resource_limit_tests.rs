use qubit_budget::LimitExceeded;
use qubit_budget::ResourceLimit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    Bytes,
}

#[test]
fn test_check_accepts_values_through_the_inclusive_maximum() {
    let limit = ResourceLimit::new(4);
    assert_eq!(limit.maximum(), 4);
    assert_eq!(limit.check(TestResource::Bytes, 0), Ok(()));
    assert_eq!(limit.check(TestResource::Bytes, 4), Ok(()));
}

#[test]
fn test_check_returns_exact_resource_limit_and_observation() {
    let limit = ResourceLimit::new(4);
    let error = limit
        .check(TestResource::Bytes, 5)
        .expect_err("five bytes must exceed a four-byte limit");
    assert_eq!(error.resource(), &TestResource::Bytes);
    assert_eq!(error.limit(), limit);
    assert_eq!(error.observed(), 5);
    assert_eq!(error.into_resource(), TestResource::Bytes);
}

#[test]
fn test_maximum_u64_is_a_finite_limit() {
    let limit = ResourceLimit::new(u64::MAX);
    assert_eq!(limit.check(TestResource::Bytes, u64::MAX), Ok(()));
}

#[test]
fn test_limit_exceeded_is_a_concrete_error() {
    let error =
        LimitExceeded::new(TestResource::Bytes, ResourceLimit::new(1), 2);
    assert!(error.to_string().contains("observed 2"));
}
