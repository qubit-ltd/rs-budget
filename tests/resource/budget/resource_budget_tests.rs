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
use qubit_budget::BudgetGroupError;
use qubit_budget::InsufficientBudgetError;
use qubit_budget::MeasuredBudgetError;
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
    budget.check_available(5).expect("the exact maximum should fit");
    assert_eq!(budget.remaining(), 5);
}

#[test]
fn test_failed_consume_is_atomic_and_reports_exact_facts() {
    let mut budget = ResourceBudget::new(TestResource::Bytes, 5_u64);
    budget.try_consume(2).expect("initial consumption should fit");
    assert!(matches!(
        budget.try_consume(4),
        Err(InsufficientBudgetError {
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
    let mut budget: ResourceBudget<TestResource, usize> = ResourceBudget::new(TestResource::Bytes, 5_usize);
    budget.try_consume(2_usize).expect("two bytes should fit");
    assert_eq!(budget.remaining(), 3_usize);
}

#[test]
fn test_budget_converts_usize_and_u64_consumption_measurements() {
    let mut budget = ResourceBudget::new(TestResource::Bytes, 5_u8);

    budget.try_consume_usize(2).expect("the usize measurement should fit");
    budget
        .check_available_u64(3)
        .expect("the exact u64 measurement should fit");
    assert_eq!(budget.remaining(), 3);
    assert!(matches!(
        budget.try_consume_u64(4),
        Err(MeasuredBudgetError::Budget(BudgetError::Insufficient {
            resource: TestResource::Bytes,
            limit: 5,
            remaining: 3,
            requested: 4,
        }))
    ));
    assert_eq!(budget.remaining(), 3);
    assert!(matches!(
        budget.try_consume_usize(usize::from(u8::MAX) + 1),
        Err(MeasuredBudgetError::Quantity { .. })
    ));
}

#[test]
fn test_check_available_usize_preserves_budget_state() {
    let budget = ResourceBudget::new(TestResource::Bytes, 5_u8);
    budget
        .check_available_usize(5)
        .expect("the exact native request should fit");
    assert_eq!(budget.remaining(), 5);
    assert!(matches!(
        budget.check_available_usize(usize::from(u8::MAX) + 1),
        Err(MeasuredBudgetError::Quantity { .. })
    ));
}

#[test]
fn test_u64_checks_and_resource_accessor_cover_success_paths() {
    let mut budget = ResourceBudget::new(TestResource::Bytes, 8_u8);
    assert_eq!(budget.resource(), &TestResource::Bytes);
    assert_eq!(budget.resource_limit().maximum(), 8);
    assert_eq!(budget.limit(), 8);
    budget.check_available_u64(3).expect("the u64 request should fit");
    budget.try_consume_u64(3).expect("the u64 request should be consumed");
    assert_eq!(budget.used(), 3);
    assert_eq!(budget.consume_available(10), 5);
    assert_eq!(budget.remaining(), 0);
}

#[test]
fn test_u64_quantity_budget_uses_u64_adapters() {
    let mut budget = ResourceBudget::new(TestResource::Bytes, 8_u64);
    budget.check_available_u64(3).expect("the u64 request should fit");
    budget.try_consume_u64(3).expect("the u64 request should be consumed");
    assert_eq!(budget.used(), 3);
}

#[test]
fn test_budget_can_be_rebuilt_from_a_resource_limit() {
    let budget = ResourceBudget::from_limit(ResourceLimit::new(TestResource::Bytes, 5_u64));
    assert_eq!(budget.limit(), 5);
    assert_eq!(budget.remaining(), 5);
}

#[test]
fn test_group_consume_charges_every_budget_after_all_checks_pass() {
    let mut local = ResourceBudget::new(TestResource::Bytes, 5_u64);
    let mut aggregate = ResourceBudget::new(TestResource::Bytes, 8_u64);

    ResourceBudget::try_consume_group(&mut [&mut local, &mut aggregate], 3)
        .expect("both budgets should accept three bytes");

    assert_eq!(local.remaining(), 2);
    assert_eq!(aggregate.remaining(), 5);
}

#[test]
fn test_group_consume_does_not_charge_any_budget_when_later_check_fails() {
    let mut local = ResourceBudget::new(TestResource::Bytes, 5_u64);
    let mut aggregate = ResourceBudget::new(TestResource::Bytes, 2_u64);

    let error = ResourceBudget::try_consume_group(&mut [&mut local, &mut aggregate], 3)
        .expect_err("the aggregate budget should reject three bytes");

    assert_eq!(error.index(), 1);
    assert!(matches!(
        error.source_error(),
        InsufficientBudgetError {
            resource: TestResource::Bytes,
            limit: 2,
            remaining: 2,
            requested: 3,
        }
    ));
    assert_eq!(local.remaining(), 5);
    assert_eq!(aggregate.remaining(), 2);
}

#[test]
fn test_group_error_exposes_the_failing_index_and_budget_error() {
    let mut first = ResourceBudget::new(TestResource::Bytes, 1_u64);
    let mut second = ResourceBudget::new(TestResource::Bytes, 1_u64);

    let error: BudgetGroupError<TestResource> = ResourceBudget::try_consume_group(&mut [&mut first, &mut second], 2)
        .expect_err("the first budget should reject two bytes");

    assert_eq!(error.index(), 0);
    assert_eq!(error.into_source_error().requested(), 2);
    assert_eq!(first.remaining(), 1);
    assert_eq!(second.remaining(), 1);
}
