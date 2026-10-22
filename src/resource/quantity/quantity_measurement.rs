// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines exact measurements and proven lower bounds that cannot fit a
//! resource quantity.

use std::fmt;

/// A native unsigned measurement or proven lower bound that cannot fit a
/// resource quantity.
///
/// # Examples
///
/// ```
/// use qubit_budget::QuantityMeasurement;
///
/// assert_eq!(QuantityMeasurement::U64(42).to_string(), "42");
/// assert_eq!(QuantityMeasurement::AtLeastU64(42).to_string(), "at least 42");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantityMeasurement {
    /// Measurement supplied by a Rust container or string length.
    Usize(
        /// Original machine-sized measurement.
        usize,
    ),
    /// Measurement supplied by an API with a stable 64-bit quantity.
    U64(
        /// Original 64-bit measurement.
        u64,
    ),
    /// A proven lower bound that cannot fit the selected resource quantity.
    AtLeastU64(
        /// Smallest value guaranteed by the measurement.
        u64,
    ),
}

impl fmt::Display for QuantityMeasurement {
    /// Formats the native measurement without changing its numeric value.
    ///
    /// # Parameters
    ///
    /// * `formatter` - Formatter receiving the decimal measurement.
    ///
    /// # Returns
    ///
    /// Returns the result of writing the measurement.
    ///
    /// # Errors
    ///
    /// Returns [`std::fmt::Error`] when the formatter rejects the output.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usize(value) => value.fmt(formatter),
            Self::U64(value) => value.fmt(formatter),
            Self::AtLeastU64(value) => write!(formatter, "at least {value}"),
        }
    }
}
