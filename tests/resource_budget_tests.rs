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
use qubit_budget::BudgetError;
use qubit_budget::ResourceBudget;
use qubit_budget::ResourceLimit;
use qubit_budget::ResourceQuantity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    Bytes,
}

const BYTE_BUDGET: ResourceBudget<&str> = ResourceBudget::new("body", 8_u64);

#[test]
fn test_new_is_const() {
    assert_eq!(BYTE_BUDGET.remaining(), 8);
}

#[test]
fn test_try_consume_decreases_remaining_and_increases_used() {
    let mut budget = ResourceBudget::new(TestResource::Bytes, 5_u64);
    budget.try_consume(2).expect("two bytes should fit");
    assert_eq!(budget.remaining(), 3);
    assert_eq!(budget.used(), 2);
}

#[test]
fn test_check_available_does_not_mutate_the_budget() {
    let budget = ResourceBudget::new(TestResource::Bytes, 5_u64);
    budget
        .check_available(5)
        .expect("the exact maximum should fit");
    assert_eq!(budget.remaining(), 5);
}

#[test]
fn test_failed_consume_is_atomic_and_reports_exact_facts() {
    let mut budget = ResourceBudget::new(TestResource::Bytes, 5_u64);
    budget
        .try_consume(2)
        .expect("initial consumption should fit");
    assert!(matches!(
        budget.try_consume(4),
        Err(BudgetError::Insufficient {
            resource: TestResource::Bytes,
            limit: 5,
            remaining: 3,
            requested: 4,
        })
    ));
    assert_eq!(budget.remaining(), 3);
}

#[test]
fn test_consume_available_returns_only_the_consumed_amount() {
    let mut budget = ResourceBudget::new(TestResource::Bytes, 5_u64);
    assert_eq!(budget.consume_available(7), 5);
    assert_eq!(budget.remaining(), 0);
    assert_eq!(budget.used(), 5);
    assert_eq!(budget.consume_available(1), 0);
}

#[test]
fn test_budget_accessors_preserve_resource_and_limit() {
    let budget = ResourceBudget::new(TestResource::Bytes, 5_u64);
    assert_eq!(budget.resource(), &TestResource::Bytes);
    assert_eq!(budget.limit(), 5_u64);
    assert_eq!(budget.resource_limit().resource(), &TestResource::Bytes);
    assert_eq!(budget.resource_limit().maximum(), 5_u64);
}

#[test]
fn test_from_limit_preserves_the_resource_limit() {
    let limit = ResourceLimit::new(TestResource::Bytes, 5_u64);
    let budget = ResourceBudget::from_limit(limit);
    assert_eq!(budget.resource_limit(), &limit);
}

proptest! {
    #[test]
    fn test_operations_preserve_remaining_and_used_invariants(
        maximum in 0_u64..=256,
        requests in prop::collection::vec(0_u64..=512, 0..64),
    ) {
        let mut budget = ResourceBudget::new(TestResource::Bytes, maximum);
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

#[test]
fn test_budget_accepts_usize_quantities_without_conversion() {
    fn assert_quantity<Q: ResourceQuantity>() {}

    assert_quantity::<usize>();
    let mut budget: ResourceBudget<TestResource, usize> =
        ResourceBudget::new(TestResource::Bytes, 5_usize);
    budget.try_consume(2_usize).expect("two bytes should fit");
    assert_eq!(budget.remaining(), 3_usize);
}
