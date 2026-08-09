// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::ResourceLimit;
use qubit_budget::ResourcePool;
use qubit_budget::ResourcePoolError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    OpenFiles,
}

#[test]
fn test_exhausted_error_exposes_available_and_requested() {
    let mut pool =
        ResourcePool::new(TestResource::OpenFiles, ResourceLimit::new(1));
    pool.try_acquire(1).expect("one unit should fit");
    let error = pool
        .try_acquire(1)
        .expect_err("one more unit should be exhausted");
    assert_eq!(error.resource(), &TestResource::OpenFiles);
    assert_eq!(error.limit(), ResourceLimit::new(1));
    assert_eq!(error.available(), Some(0));
    assert_eq!(error.in_use(), None);
    assert_eq!(error.requested(), 1);
}

#[test]
fn test_invalid_release_error_exposes_in_use_and_requested() {
    let mut pool =
        ResourcePool::new(TestResource::OpenFiles, ResourceLimit::new(1));
    let error = pool.release(1).expect_err("nothing is held yet");
    assert!(matches!(error, ResourcePoolError::InvalidRelease { .. }));
    assert_eq!(error.resource(), &TestResource::OpenFiles);
    assert_eq!(error.limit(), ResourceLimit::new(1));
    assert_eq!(error.available(), None);
    assert_eq!(error.in_use(), Some(0));
    assert_eq!(error.requested(), 1);
}

#[test]
fn test_pool_errors_can_be_displayed_and_consumed_for_the_resource() {
    let exhausted = ResourcePoolError::Exhausted {
        resource: TestResource::OpenFiles,
        limit: ResourceLimit::new(1),
        available: 0,
        requested: 1,
    };
    assert!(exhausted.to_string().contains("available"));
    assert_eq!(exhausted.into_resource(), TestResource::OpenFiles);

    let invalid = ResourcePoolError::InvalidRelease {
        resource: TestResource::OpenFiles,
        limit: ResourceLimit::new(1),
        in_use: 0,
        requested: 1,
    };
    assert!(invalid.to_string().contains("released"));
    assert_eq!(invalid.into_resource(), TestResource::OpenFiles);
}
