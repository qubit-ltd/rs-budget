// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines failures converting native measurements into resource quantities.
// qubit-style: allow source-test-pair

use crate::resource::QuantityMeasurement;

/// Error returned when a native measurement cannot fit the selected quantity.
///
/// # Examples
///
/// ```
/// use qubit_budget::QuantityConversionError;
/// use qubit_budget::QuantityMeasurement;
///
/// let error = QuantityConversionError::new(QuantityMeasurement::Usize(256), "u8");
/// assert_eq!(error.target(), "u8");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("measurement {measurement} cannot be represented by {target}")]
pub struct QuantityConversionError {
    /// Original native measurement retained without truncation.
    measurement: QuantityMeasurement,
    /// Name of the selected resource quantity type.
    target: &'static str,
}

impl QuantityConversionError {
    /// Creates one quantity representation failure.
    ///
    /// # Parameters
    ///
    /// * `measurement` - Original native measurement that did not fit.
    /// * `target` - Name of the selected resource quantity type.
    ///
    /// # Returns
    ///
    /// A failure retaining both the original measurement and target type.
    #[inline(always)]
    #[must_use]
    pub const fn new(measurement: QuantityMeasurement, target: &'static str) -> Self {
        Self { measurement, target }
    }

    /// Returns the native measurement that could not be represented.
    ///
    /// # Returns
    ///
    /// Returns the native measurement that could not be represented.
    #[must_use]
    #[inline(always)]
    pub const fn measurement(&self) -> QuantityMeasurement {
        self.measurement
    }

    /// Returns the selected resource quantity type name.
    ///
    /// # Returns
    ///
    /// Returns the selected resource quantity type name.
    #[must_use]
    #[inline(always)]
    pub const fn target(&self) -> &'static str {
        self.target
    }
}
