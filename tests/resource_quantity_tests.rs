// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the supported resource quantity types.

use qubit_budget::ResourceQuantity;

#[test]
fn test_unsigned_quantities_provide_zero_and_one() {
    assert_eq!(<u8 as ResourceQuantity>::ZERO, 0);
    assert_eq!(<usize as ResourceQuantity>::ONE, 1);
}
