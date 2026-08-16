// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for resource-pool release errors.

use qubit_budget::ResourceReleaseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestResource {
    Readers,
}

#[test]
fn test_invalid_release_error_exposes_release_facts() {
    let error = ResourceReleaseError::InvalidRelease {
        resource: TestResource::Readers,
        limit: 3_usize,
        in_use: 1,
        requested: 2,
    };

    assert_eq!(error.resource(), &TestResource::Readers);
    assert_eq!(error.limit(), 3);
    assert_eq!(error.in_use(), 1);
    assert_eq!(error.requested(), 2);
    assert_eq!(error.into_resource(), TestResource::Readers);
}
