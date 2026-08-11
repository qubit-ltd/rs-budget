// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the supported resource quantity types.

use qubit_budget::QuantityMeasurement;
use qubit_budget::ResourceQuantity;

#[test]
fn test_unsigned_quantities_provide_checked_addition() {
    assert_eq!(<u8 as ResourceQuantity>::checked_add(2, 3), Some(5));
    assert_eq!(<usize as ResourceQuantity>::checked_add(2, 3), Some(5));
    assert_eq!(<u8 as ResourceQuantity>::checked_add(u8::MAX, 1), None,);
}

#[test]
fn test_unsigned_quantities_provide_zero_and_one() {
    assert_eq!(<u8 as ResourceQuantity>::ZERO, 0);
    assert_eq!(<usize as ResourceQuantity>::ONE, 1);
}

#[test]
fn test_unsigned_quantities_convert_native_measurements_without_truncation() {
    assert_eq!(usize::try_from_usize(7), Ok(7));
    assert_eq!(u64::try_from_usize(7), Ok(7));
    assert_eq!(u128::try_from_u64(u64::MAX), Ok(u128::from(u64::MAX)));

    let error = u8::try_from_usize(usize::from(u8::MAX) + 1)
        .expect_err("u8 must reject an oversized usize measurement");
    assert_eq!(error.measurement(), QuantityMeasurement::Usize(256));
    assert_eq!(error.target(), "u8");
}
