// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::time::Duration;

use qubit_budget::TimeBudget;
use qubit_budget::TimeBudgetError;
use qubit_clock::ManualMonotonicClock;
use qubit_clock::MonotonicClock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    TotalElapsed,
}

#[test]
fn test_for_duration_counts_all_monotonic_elapsed_time() {
    let clock = ManualMonotonicClock::new_shared();
    let budget = TimeBudget::for_duration(
        TestResource::TotalElapsed,
        clock.clone(),
        Duration::from_secs(10),
    )
    .expect("deadline should be representable");
    clock
        .advance(Duration::from_secs(4))
        .expect("manual clock should advance");
    assert_eq!(
        budget.elapsed().expect("elapsed time should be valid"),
        Duration::from_secs(4)
    );
    assert_eq!(
        budget.remaining().expect("remaining time should be valid"),
        Duration::from_secs(6)
    );
}

#[test]
fn test_deadline_is_expired_at_the_exact_boundary() {
    let clock = ManualMonotonicClock::new_shared();
    let budget = TimeBudget::for_duration(
        TestResource::TotalElapsed,
        clock.clone(),
        Duration::from_secs(5),
    )
    .expect("deadline should be representable");
    clock
        .advance(Duration::from_secs(5))
        .expect("manual clock should advance");
    assert_eq!(
        budget.remaining().expect("remaining should saturate"),
        Duration::ZERO
    );
    assert!(
        budget
            .is_expired()
            .expect("expiration check should succeed")
    );
    assert!(matches!(
        budget.check(),
        Err(TimeBudgetError::Expired { .. })
    ));
}

#[test]
fn test_check_after_rejects_an_operation_reaching_the_deadline() {
    let clock = ManualMonotonicClock::new_shared();
    let budget = TimeBudget::for_duration(
        TestResource::TotalElapsed,
        clock.clone(),
        Duration::from_secs(5),
    )
    .expect("deadline should be representable");
    let error = budget
        .check_after(Duration::from_secs(5))
        .expect_err("reaching the deadline must be rejected");
    assert!(matches!(
        error,
        TimeBudgetError::WouldExpire { requested, .. }
            if requested == Duration::from_secs(5)
    ));
}

#[test]
fn test_check_after_allows_an_operation_strictly_before_the_deadline() {
    let clock = ManualMonotonicClock::new_shared();
    let budget = TimeBudget::for_duration(
        TestResource::TotalElapsed,
        clock.clone(),
        Duration::from_secs(5),
    )
    .expect("deadline should be representable");
    budget
        .check_after(Duration::from_secs(4))
        .expect("four seconds should fit before the deadline");
}

#[test]
fn test_until_rejects_a_foreign_clock_domain() {
    let first = ManualMonotonicClock::new_shared();
    let second = ManualMonotonicClock::new_shared();
    let deadline = first
        .deadline_after(Duration::from_secs(1))
        .expect("deadline should be representable");
    let error =
        TimeBudget::until(TestResource::TotalElapsed, second.clone(), deadline)
            .expect_err("a foreign deadline must be rejected");
    assert!(matches!(error, TimeBudgetError::Clock { .. }));
}

#[test]
fn test_until_success_exposes_resource_and_fixed_instants() {
    let clock = ManualMonotonicClock::new_shared();
    let deadline = clock
        .deadline_after(Duration::from_secs(5))
        .expect("deadline should be representable");
    let budget =
        TimeBudget::until(TestResource::TotalElapsed, clock.clone(), deadline)
            .expect("same-domain deadline should be accepted");
    assert_eq!(budget.resource(), &TestResource::TotalElapsed);
    assert_eq!(budget.started_at(), clock.now());
    assert_eq!(budget.deadline(), deadline);
    assert!(!budget.is_expired().expect("deadline should not be expired"));
    budget.check().expect("deadline should still be open");
}

#[test]
fn test_check_after_reports_expired_and_clock_overflow_paths() {
    let expired_clock = ManualMonotonicClock::new_shared();
    let expired_budget = TimeBudget::for_duration(
        TestResource::TotalElapsed,
        expired_clock.clone(),
        Duration::ZERO,
    )
    .expect("zero deadline should be representable");
    assert!(matches!(
        expired_budget.check_after(Duration::ZERO),
        Err(TimeBudgetError::Expired { .. })
    ));

    let overflow_clock = ManualMonotonicClock::new_shared();
    let near_max = Duration::MAX
        .checked_sub(Duration::from_nanos(1))
        .expect("maximum duration should contain one nanosecond");
    overflow_clock
        .advance(near_max)
        .expect("near-maximum instant should be representable");
    let deadline = overflow_clock
        .deadline_after(Duration::from_nanos(1))
        .expect("maximum instant should be representable");
    let overflow_budget =
        TimeBudget::until(TestResource::TotalElapsed, overflow_clock, deadline)
            .expect("same-domain deadline should be accepted");
    assert!(matches!(
        overflow_budget.check_after(Duration::from_nanos(2)),
        Err(TimeBudgetError::Clock { .. })
    ));
}

#[test]
fn test_elapsed_time_includes_waiting_and_backoff() {
    let clock = ManualMonotonicClock::new_shared();
    let budget = TimeBudget::for_duration(
        TestResource::TotalElapsed,
        clock.clone(),
        Duration::from_secs(10),
    )
    .expect("deadline should be representable");
    clock
        .advance(Duration::from_secs(2))
        .expect("operation should advance time");
    clock
        .advance(Duration::from_secs(3))
        .expect("waiting should advance time");
    clock
        .advance(Duration::from_secs(4))
        .expect("backoff should advance time");
    assert_eq!(
        budget.remaining().expect("remaining should be valid"),
        Duration::from_secs(1)
    );
}
