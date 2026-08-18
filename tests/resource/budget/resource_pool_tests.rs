// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use proptest::prelude::any;
use proptest::prelude::prop;
use proptest::prelude::prop_assert_eq;
use proptest::prelude::proptest;
use qubit_budget::InsufficientBudgetError;
use qubit_budget::ResourceLimit;
use qubit_budget::ResourcePool;
use qubit_budget::ResourceQuantity;
use qubit_budget::ResourceReleaseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    OpenFiles,
}

const OPEN_FILE_POOL: ResourcePool<&str> = ResourcePool::new("open-files", 3_u64);

#[test]
fn test_new_is_const() {
    assert_eq!(OPEN_FILE_POOL.capacity(), 3);
}

#[test]
fn test_release_can_happen_in_another_context_and_in_parts() {
    fn close_one(pool: &mut ResourcePool<&str>) {
        pool.release(1).expect("one held unit should be releasable");
    }

    let mut pool = OPEN_FILE_POOL;
    pool.try_acquire(3).expect("capacity should fit");
    close_one(&mut pool);
    assert_eq!(pool.in_use(), 2);
    pool.release(2).expect("the remaining units should be releasable");
    assert_eq!(pool.in_use(), 0);
}

fn acquire_then_release(pool: &mut ResourcePool<TestResource>, amount: u64) -> Result<(), String> {
    pool.try_acquire(amount).map_err(|error| error.to_string())?;
    pool.release(amount).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn test_release_makes_capacity_reusable() {
    let mut pool = ResourcePool::new(TestResource::OpenFiles, 2_u64);
    pool.try_acquire(2).expect("capacity should be acquirable");
    pool.release(1).expect("one held unit should be releasable");
    pool.try_acquire(1).expect("released capacity should be reusable");
    assert_eq!(pool.available(), 0);
    assert_eq!(pool.in_use(), 2);
}

#[test]
fn test_acquire_and_release_have_distinct_error_types() {
    let mut pool = ResourcePool::new(TestResource::OpenFiles, 2_u64);
    acquire_then_release(&mut pool, 1).expect("both operations should share one error type");
}

#[test]
fn test_acquire_reports_exhaustion_without_mutation() {
    let mut pool = ResourcePool::new(TestResource::OpenFiles, 2_u64);
    pool.try_acquire(2).expect("the maximum should be acquirable");
    let error = pool.try_acquire(1).expect_err("one more unit should be exhausted");
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
fn test_invalid_release_is_atomic() {
    let mut pool = ResourcePool::new(TestResource::OpenFiles, 2_u64);
    pool.try_acquire(1).expect("one unit should fit");
    let error = pool.release(2).expect_err("cannot release more than is held");
    assert!(matches!(
        error,
        ResourceReleaseError::InvalidRelease {
            resource: TestResource::OpenFiles,
            limit: 2,
            in_use: 1,
            requested: 2,
        }
    ));
    assert_eq!(pool.available(), 1);
}

#[test]
fn test_pool_accessors_report_the_finite_capacity() {
    let pool = ResourcePool::new(TestResource::OpenFiles, 2_u64);
    assert_eq!(pool.resource(), &TestResource::OpenFiles);
    assert_eq!(pool.limit(), 2_u64);
    assert_eq!(pool.resource_limit().resource(), &TestResource::OpenFiles);
    assert_eq!(pool.resource_limit().maximum(), 2_u64);
    assert_eq!(pool.capacity(), 2);
    assert_eq!(pool.available(), 2);
    assert_eq!(pool.in_use(), 0);
}

#[test]
fn test_from_limit_preserves_the_resource_limit() {
    let limit = ResourceLimit::new(TestResource::OpenFiles, 2_u64);
    let pool = ResourcePool::from_limit(limit);
    assert_eq!(pool.resource_limit(), &limit);
}

proptest! {
    #[test]
    fn test_legal_acquire_release_sequences_preserve_pool_invariants(
        maximum in 0_u64..=128,
        operations in prop::collection::vec((any::<bool>(), 0_u16..=256), 0..64),
    ) {
        let mut pool = ResourcePool::new(TestResource::OpenFiles, maximum);
        let mut model_available = maximum;

        for (acquire, raw_amount) in operations {
            if acquire {
                let requested = u64::from(raw_amount) % (model_available + 1);
                pool.try_acquire(requested).expect("model-generated acquire fits");
                model_available -= requested;
            } else {
                let in_use = maximum - model_available;
                let requested = u64::from(raw_amount) % (in_use + 1);
                pool.release(requested).expect("model-generated release is valid");
                model_available += requested;
            }
            prop_assert_eq!(pool.available(), model_available);
            prop_assert_eq!(pool.available() + pool.in_use(), maximum);
        }
    }
}

#[test]
fn test_pool_accepts_usize_quantities_without_conversion() {
    fn assert_quantity<Q: ResourceQuantity>() {}

    assert_quantity::<usize>();
    let mut pool: ResourcePool<TestResource, usize> = ResourcePool::new(TestResource::OpenFiles, 2_usize);
    pool.try_acquire(1_usize).expect("one directory should fit");
    assert_eq!(pool.available(), 1_usize);
}
