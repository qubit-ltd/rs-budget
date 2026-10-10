// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for resource observations.

use qubit_budget::Observation;

/// Verifies exact and lower-bound observations expose their precision.
#[test]
fn test_observation_reports_exactness_and_lower_bound() {
    let exact = Observation::Exact(4_u64);
    let lower_bound = Observation::AtLeast(5_u64);

    assert_eq!(exact.exact(), Some(4), "exact observations should expose their value");
    assert_eq!(
        exact.lower_bound(),
        4,
        "exact observations should expose their lower bound"
    );
    assert_eq!(
        lower_bound.exact(),
        None,
        "lower-bound observations should not claim an exact value"
    );
    assert_eq!(
        lower_bound.lower_bound(),
        5,
        "lower-bound observations should expose their minimum value"
    );
    assert_eq!(
        lower_bound.to_string(),
        "at least 5",
        "lower-bound observations should use descriptive display text"
    );
}
