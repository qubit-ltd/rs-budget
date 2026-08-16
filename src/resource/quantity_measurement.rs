// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines native measurements that cannot fit a resource quantity.

use std::fmt;

/// A native unsigned measurement whose value could not fit a resource quantity.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantityMeasurement {
    /// Measurement supplied by a Rust container or string length.
    Usize(usize),
    /// Measurement supplied by an API with a stable 64-bit quantity.
    U64(u64),
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
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usize(value) => value.fmt(formatter),
            Self::U64(value) => value.fmt(formatter),
        }
    }
}
