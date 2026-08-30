// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for shared pools with automatically released permits.

use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;
use std::thread;

use proptest::prelude::prop;
use proptest::prelude::prop_assert_eq;
use proptest::prelude::proptest;
use qubit_budget::InsufficientBudgetError;
use qubit_budget::ManagedResourcePool;
use qubit_budget::ResourceLimit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    OpenFiles,
}

#[test]
fn test_drop_releases_acquired_capacity() {
    let pool = ManagedResourcePool::new(TestResource::OpenFiles, 2_u64);
    {
        let permit = pool.try_acquire(2).expect("capacity should fit");
        assert_eq!(permit.resource(), &TestResource::OpenFiles);
        assert_eq!(permit.amount(), 2);
        assert_eq!(pool.available(), 0);
    }
    assert_eq!(pool.available(), 2);
}

#[test]
fn test_clones_share_capacity_and_failure_is_atomic() {
    let pool = ManagedResourcePool::new(TestResource::OpenFiles, 2_u64);
    let clone = pool.clone();
    let _permit = pool.try_acquire(2).expect("capacity should fit");
    let error = clone.try_acquire(1).expect_err("shared capacity should be exhausted");
    assert!(matches!(
        error,
        InsufficientBudgetError {
            resource: TestResource::OpenFiles,
            limit: 2,
            remaining: 0,
            requested: 1,
        }
    ));
    assert_eq!(pool.available(), 0);
}

#[test]
fn test_release_consumes_permit_and_restores_capacity() {
    let pool = ManagedResourcePool::new(TestResource::OpenFiles, 1_u64);
    pool.try_acquire(1).expect("capacity should fit").release();
    assert_eq!(pool.available(), 1);
}

#[test]
fn test_unwinding_releases_capacity() {
    let pool = ManagedResourcePool::new(TestResource::OpenFiles, 1_u64);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let _permit = pool.try_acquire(1).expect("capacity should fit");
        panic!("exercise permit drop during unwinding");
    }));
    assert!(result.is_err());
    assert_eq!(pool.available(), 1);
}

#[test]
fn test_permit_can_move_to_another_thread() {
    let pool = ManagedResourcePool::new(TestResource::OpenFiles, 1_u64);
    let permit = pool.try_acquire(1).expect("capacity should fit");
    thread::spawn(move || drop(permit))
        .join()
        .expect("permit thread should finish");
    assert_eq!(pool.available(), 1);
}

#[test]
fn test_zero_quantity_preserves_pool_state() {
    let pool = ManagedResourcePool::new(TestResource::OpenFiles, 1_u64);
    let permit = pool.try_acquire(0).expect("zero should always fit");
    assert_eq!(pool.available(), 1);
    drop(permit);
    assert_eq!(pool.available(), 1);
}

#[test]
fn test_from_limit_and_accessors_preserve_configuration() {
    let limit = ResourceLimit::new(TestResource::OpenFiles, 3_u64);
    let pool = ManagedResourcePool::from_limit(limit);
    assert_eq!(pool.resource(), &TestResource::OpenFiles);
    assert_eq!(pool.resource_limit(), &limit);
    assert_eq!(pool.capacity(), 3);
    assert_eq!(pool.available(), 3);
    assert_eq!(pool.in_use(), 0);
}

proptest! {
    #[test]
    fn test_permit_sequences_preserve_pool_invariants(
        maximum in 0_u64..=64,
        amounts in prop::collection::vec(0_u64..=64, 0..32),
    ) {
        let pool = ManagedResourcePool::new(TestResource::OpenFiles, maximum);
        for raw_amount in amounts {
            let amount = raw_amount.min(pool.available());
            let permit = pool.try_acquire(amount).expect("bounded amount should fit");
            prop_assert_eq!(pool.available() + pool.in_use(), maximum);
            drop(permit);
            prop_assert_eq!(pool.available(), maximum);
            prop_assert_eq!(pool.available() + pool.in_use(), maximum);
        }
    }
}
