// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use qubit_budget::json::JsonMeasurement;

/// Verifies string measurements retain native depth and payload dimensions.
#[test]
fn test_string_measurement_retains_native_dimensions() {
    let measurement = JsonMeasurement::String {
        depth: usize::MAX,
        bytes: usize::MAX,
    };

    assert_eq!(
        measurement,
        JsonMeasurement::String {
            depth: usize::MAX,
            bytes: usize::MAX,
        },
    );
}
