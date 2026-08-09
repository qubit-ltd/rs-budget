// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use proptest::prelude::prop;
use proptest::prelude::prop_assert;
use proptest::prelude::prop_assert_eq;
use proptest::prelude::proptest;
use qubit_budget::ResourceBudget;
use qubit_budget::ResourceLimit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    Bytes,
}

const BYTE_BUDGET: ResourceBudget<&str> =
    ResourceBudget::new("body", ResourceLimit::new(8));

#[test]
fn test_new_is_const() {
    assert_eq!(BYTE_BUDGET.remaining(), 8);
}

#[test]
fn test_try_consume_decreases_remaining_and_increases_used() {
    let mut budget =
        ResourceBudget::new(TestResource::Bytes, ResourceLimit::new(5));
    budget.try_consume(2).expect("two bytes should fit");
    assert_eq!(budget.remaining(), 3);
    assert_eq!(budget.used(), 2);
}

#[test]
fn test_check_available_does_not_mutate_the_budget() {
    let budget =
        ResourceBudget::new(TestResource::Bytes, ResourceLimit::new(5));
    budget
        .check_available(5)
        .expect("the exact maximum should fit");
    assert_eq!(budget.remaining(), 5);
}

#[test]
fn test_failed_consume_is_atomic_and_reports_exact_facts() {
    let mut budget =
        ResourceBudget::new(TestResource::Bytes, ResourceLimit::new(5));
    budget
        .try_consume(2)
        .expect("initial consumption should fit");
    let error = budget
        .try_consume(4)
        .expect_err("four bytes should not fit");
    assert_eq!(error.resource(), &TestResource::Bytes);
    assert_eq!(error.limit(), ResourceLimit::new(5));
    assert_eq!(error.remaining(), 3);
    assert_eq!(error.requested(), 4);
    assert_eq!(budget.remaining(), 3);
}

#[test]
fn test_error_reports_used_and_checked_attempted() {
    let mut budget = ResourceBudget::new("body", ResourceLimit::new(10));
    budget.try_consume(7).expect("seven bytes should fit");
    let error = budget
        .try_consume(5)
        .expect_err("five bytes should exceed the remainder");
    assert_eq!(error.used(), 7);
    assert_eq!(error.checked_attempted(), Some(12));
}

#[test]
fn test_error_checked_attempted_reports_overflow() {
    let mut budget = ResourceBudget::new("body", ResourceLimit::new(u64::MAX));
    budget
        .try_consume(u64::MAX)
        .expect("the maximum amount should fit");
    let error = budget
        .try_consume(1)
        .expect_err("an exhausted budget should reject more input");
    assert_eq!(error.used(), u64::MAX);
    assert_eq!(error.checked_attempted(), None);
}

#[test]
fn test_consume_available_returns_only_the_consumed_amount() {
    let mut budget =
        ResourceBudget::new(TestResource::Bytes, ResourceLimit::new(5));
    assert_eq!(budget.consume_available(7), 5);
    assert_eq!(budget.remaining(), 0);
    assert_eq!(budget.used(), 5);
    assert_eq!(budget.consume_available(1), 0);
}

#[test]
fn test_budget_accessors_preserve_resource_and_limit() {
    let budget =
        ResourceBudget::new(TestResource::Bytes, ResourceLimit::new(5));
    assert_eq!(budget.resource(), &TestResource::Bytes);
    assert_eq!(budget.limit(), ResourceLimit::new(5));
}

proptest! {
    #[test]
    fn test_operations_preserve_remaining_and_used_invariants(
        maximum in 0_u64..=256,
        requests in prop::collection::vec(0_u64..=512, 0..64),
    ) {
        let mut budget = ResourceBudget::new(TestResource::Bytes, ResourceLimit::new(maximum));
        for requested in requests {
            let before = budget.remaining();
            let result = budget.try_consume(requested);
            if result.is_err() {
                prop_assert_eq!(budget.remaining(), before);
            }
            prop_assert!(budget.remaining() <= maximum);
            prop_assert_eq!(budget.used() + budget.remaining(), maximum);
        }
    }
}
