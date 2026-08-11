use qubit_budget::ResourceLimit;
use qubit_budget::StringLimits;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    Bytes,
}

#[test]
fn test_check_uses_utf8_bytes_and_check_name() {
    let limits =
        StringLimits::empty().with_utf8_bytes_limit(ResourceLimit::new(TestResource::Bytes, 2));
    let error = limits.check("中").expect_err("three bytes exceed two");
    assert_eq!(error.exact_observed(), Some(3));
}

#[test]
fn test_empty_limits_accept_any_string() {
    StringLimits::<TestResource>::empty()
        .check("arbitrary")
        .expect("unconfigured string limits must accept the value");
}
