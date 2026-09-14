// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_budget::MeasuredBudgetError;
use qubit_budget::ResourceLimit;
use qubit_budget::StringLimits;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    Bytes,
}

#[test]
fn test_check_uses_utf8_bytes_and_check_name() {
    let limits = StringLimits::builder()
        .utf8_bytes_limit(ResourceLimit::new(TestResource::Bytes, 2_u64))
        .build();
    let error = limits.check("中").expect_err("three bytes exceed two");
    assert_eq!(
        error
            .budget_error()
            .and_then(|budget_error| budget_error.exact_observed()),
        Some(3)
    );
}

#[test]
fn test_empty_limits_accept_any_string() {
    StringLimits::<TestResource>::builder()
        .build()
        .check("arbitrary")
        .expect("unconfigured string limits must accept the value");
}

#[test]
fn test_string_limits_support_usize_quantities() {
    let limits = StringLimits::<TestResource, usize>::builder()
        .utf8_bytes_limit(ResourceLimit::new(TestResource::Bytes, 3))
        .build();

    limits
        .check("abc")
        .expect("three bytes should fit the usize limit");
    let error = limits.check("abcd").expect_err("four bytes exceed three");
    assert_eq!(
        error.budget_error().and_then(|error| error.maximum()),
        Some(3)
    );
}

#[test]
fn test_string_limits_reject_unrepresentable_measurements() {
    let limits = StringLimits::<TestResource, u8>::builder()
        .utf8_bytes_limit(ResourceLimit::new(TestResource::Bytes, u8::MAX))
        .build();
    let text = "x".repeat(usize::from(u8::MAX) + 1);

    assert!(matches!(
        limits.check(&text),
        Err(MeasuredBudgetError::Quantity { .. })
    ));
}

#[test]
fn test_string_limits_default_is_unconfigured() {
    StringLimits::<TestResource>::default()
        .check("default")
        .expect("default limits accept every string");
}
