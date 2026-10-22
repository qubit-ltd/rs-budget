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
use std::sync::Arc;
use std::sync::Barrier;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread;

use proptest::prelude::prop;
use proptest::prelude::prop_assert;
use proptest::prelude::prop_assert_eq;
use proptest::prelude::proptest;
use qubit_budget::InsufficientBudgetError;
use qubit_budget::ManagedResourcePermit;
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

/// Verifies contending acquisitions never exceed capacity, failed attempts do
/// not corrupt the pool, and dropping all permits restores full capacity.
#[test]
fn test_contending_acquisitions_preserve_capacity() {
    const CAPACITY: usize = 3;
    const THREADS: usize = 8;

    let pool = ManagedResourcePool::new(TestResource::OpenFiles, CAPACITY);
    let start = Arc::new(Barrier::new(THREADS + 1));
    let acquired = Arc::new(Barrier::new(THREADS + 1));
    let release = Arc::new(Barrier::new(THREADS + 1));
    let held = Arc::new(AtomicUsize::new(0));
    let maximum_held = Arc::new(AtomicUsize::new(0));
    let successes = Arc::new(AtomicUsize::new(0));
    let failures = Arc::new(AtomicUsize::new(0));
    let mut workers = Vec::with_capacity(THREADS);

    for _ in 0..THREADS {
        let pool = pool.clone();
        let start = Arc::clone(&start);
        let acquired = Arc::clone(&acquired);
        let release = Arc::clone(&release);
        let held = Arc::clone(&held);
        let maximum_held = Arc::clone(&maximum_held);
        let successes = Arc::clone(&successes);
        let failures = Arc::clone(&failures);
        workers.push(thread::spawn(move || {
            start.wait();
            let permit = match pool.try_acquire(1) {
                Ok(permit) => {
                    successes.fetch_add(1, Ordering::SeqCst);
                    let current = held.fetch_add(1, Ordering::SeqCst) + 1;
                    maximum_held.fetch_max(current, Ordering::SeqCst);
                    Some(permit)
                }
                Err(_) => {
                    failures.fetch_add(1, Ordering::SeqCst);
                    None
                }
            };
            acquired.wait();
            release.wait();
            let acquired_permit = permit.is_some();
            drop(permit);
            if acquired_permit {
                held.fetch_sub(1, Ordering::SeqCst);
            }
        }));
    }

    start.wait();
    acquired.wait();
    assert_eq!(successes.load(Ordering::SeqCst), CAPACITY);
    assert_eq!(failures.load(Ordering::SeqCst), THREADS - CAPACITY);
    assert_eq!(held.load(Ordering::SeqCst), CAPACITY);
    assert!(maximum_held.load(Ordering::SeqCst) <= CAPACITY);
    assert_eq!(pool.available(), 0);
    assert_eq!(pool.in_use(), CAPACITY);

    release.wait();
    for worker in workers {
        worker.join().expect("contention worker should finish");
    }
    assert_eq!(held.load(Ordering::SeqCst), 0);
    assert_eq!(pool.available(), CAPACITY);
    assert_eq!(pool.in_use(), 0);
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
        operations in prop::collection::vec((0_u8..4, 0_u64..=96, 0_usize..64), 1..64),
    ) {
        let pool = ManagedResourcePool::new(TestResource::OpenFiles, maximum);
        let other = pool.clone();
        let mut held: Vec<ManagedResourcePermit<TestResource>> = Vec::new();
        for (operation, amount, index) in operations {
            let before_used: u64 = held.iter().map(ManagedResourcePermit::amount).sum();
            let before_available = maximum - before_used;
            match operation {
                0 | 1 => {
                    let handle = if operation == 0 { &pool } else { &other };
                    match handle.try_acquire(amount) {
                        Ok(permit) => {
                            prop_assert!(amount <= before_available);
                            prop_assert_eq!(permit.amount(), amount);
                            held.push(permit);
                        }
                        Err(error) => {
                            prop_assert!(amount > before_available);
                            prop_assert_eq!(error.resource(), &TestResource::OpenFiles);
                            prop_assert_eq!(error.limit(), maximum);
                            prop_assert_eq!(error.remaining(), before_available);
                            prop_assert_eq!(error.requested(), amount);
                            prop_assert_eq!(pool.available(), before_available);
                        }
                    }
                }
                2 if !held.is_empty() => {
                    let selected = index % held.len();
                    drop(held.swap_remove(selected));
                }
                3 if !held.is_empty() => {
                    let selected = index % held.len();
                    held.swap_remove(selected).release();
                }
                _ => {}
            }
            let used: u64 = held.iter().map(ManagedResourcePermit::amount).sum();
            prop_assert!(used <= maximum);
            prop_assert_eq!(pool.in_use(), used);
            prop_assert_eq!(pool.available(), maximum - used);
            prop_assert_eq!(other.available(), maximum - used);
        }
        drop(held);
        prop_assert_eq!(pool.available(), maximum);
        prop_assert_eq!(pool.in_use(), 0);
    }
}

#[test]
fn test_miri_managed_resource_permit_lifecycle() {
    let pool = ManagedResourcePool::new(TestResource::OpenFiles, 3_u64);
    let other = pool.clone();
    let first = pool.try_acquire(1).expect("one fits");
    let second = other.try_acquire(2).expect("two fit");
    assert_eq!(pool.available(), 0);
    let error = other.try_acquire(1).expect_err("shared pool exhausted");
    assert_eq!(error.remaining(), 0);
    first.release();
    assert_eq!(other.available(), 1);
    drop(second);
    assert_eq!(pool.available(), 3);
    let last = other.try_acquire(3).expect("capacity reusable");
    drop(pool);
    drop(other);
    assert_eq!(last.amount(), 3);
    assert_eq!(last.resource(), &TestResource::OpenFiles);
    drop(last);
}
