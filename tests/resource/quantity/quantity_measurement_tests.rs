// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::fmt::Write;

use qubit_budget::QuantityConversionError;
use qubit_budget::QuantityMeasurement;

#[test]
fn test_quantity_measurement_formats_each_native_variant() {
    let mut text = String::new();
    write!(
        &mut text,
        "{} {}",
        QuantityMeasurement::Usize(1),
        QuantityMeasurement::U64(2),
    )
    .expect("formatting should succeed");
    assert_eq!(text, "1 2");
}

#[test]
fn test_lower_bound_measurement_is_explicit_and_converts() {
    let measurement = QuantityMeasurement::AtLeastU64(4097);
    assert_eq!(measurement, QuantityMeasurement::AtLeastU64(4097));
    assert_eq!(measurement.to_string(), "at least 4097");

    let error = QuantityConversionError::new(measurement, "u8");
    assert_eq!(error.measurement(), measurement);
    assert_eq!(error.target(), "u8");
}
