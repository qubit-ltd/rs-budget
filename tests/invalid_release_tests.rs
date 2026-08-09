// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Integration tests for invalid capacity releases.

use qubit_budget::InvalidRelease;

/// Verifies that invalid release facts expose both capacity values.
#[test]
fn exposes_release_facts() {
    let error = InvalidRelease::new(3, 5);
    assert_eq!(error.releasable(), 3);
    assert_eq!(error.requested(), 5);
}
