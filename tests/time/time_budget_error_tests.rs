// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::error::Error;
use std::time::Duration;

use qubit_budget::TimeBudget;
use qubit_budget::TimeBudgetError;
use qubit_clock::ManualMonotonicClock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    TotalElapsed,
}

#[test]
fn test_expired_error_exposes_resource_deadline_and_now() {
    let clock = ManualMonotonicClock::new_shared();
    let budget = TimeBudget::for_duration(
        TestResource::TotalElapsed,
        clock.clone(),
        Duration::ZERO,
    )
    .expect("zero deadline should be representable");
    let error = budget
        .check()
        .expect_err("zero deadline is already reached");
    assert!(matches!(error, TimeBudgetError::Expired { .. }));
    assert_eq!(error.resource(), &TestResource::TotalElapsed);
    assert!(error.deadline().is_some());
    assert!(error.now().is_some());
    assert_eq!(error.requested(), None);
}

#[test]
fn test_clock_error_exposes_the_resource_and_time_source() {
    let clock = ManualMonotonicClock::new_shared();
    clock
        .advance(Duration::MAX)
        .expect("the maximum instant should be representable");
    let error = TimeBudget::for_duration(
        TestResource::TotalElapsed,
        clock.clone(),
        Duration::from_nanos(1),
    )
    .expect_err("the maximum duration should overflow the instant");
    assert!(matches!(error, TimeBudgetError::Clock { .. }));
    assert_eq!(error.resource(), &TestResource::TotalElapsed);
    assert!(error.clock_error().is_some());
    assert_eq!(error.deadline(), None);
    assert_eq!(error.now(), None);
    assert_eq!(error.requested(), None);
    assert!(error.to_string().contains("failed:"));
    assert!(Error::source(&error).is_some());
    assert_eq!(error.into_resource(), TestResource::TotalElapsed);
}

#[test]
fn test_would_expire_error_exposes_fields_and_display() {
    let clock = ManualMonotonicClock::new_shared();
    let budget = TimeBudget::for_duration(
        TestResource::TotalElapsed,
        clock,
        Duration::from_secs(5),
    )
    .expect("deadline should be representable");
    let error = budget
        .check_after(Duration::from_secs(5))
        .expect_err("the exact deadline is not an allowed finish time");
    assert_eq!(error.resource(), &TestResource::TotalElapsed);
    assert!(error.clock_error().is_none());
    assert!(error.deadline().is_some());
    assert!(error.now().is_some());
    assert_eq!(error.requested(), Some(Duration::from_secs(5)));
    assert!(error.to_string().contains("cannot fit"));
    assert!(Error::source(&error).is_none());
    assert_eq!(error.into_resource(), TestResource::TotalElapsed);
}
