// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Builds coefficient and scale limits for `BigDecimal`.

use super::BigDecimalLimits;
use super::BigIntegerLimits;
use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;

/// Builder for [`BigDecimalLimits`].
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Examples
///
/// ```
/// use qubit_budget::BigDecimalLimitsBuilder;
/// use qubit_budget::ResourceLimit;
///
/// let limits = BigDecimalLimitsBuilder::new()
///     .scale_magnitude_limit(ResourceLimit::new("scale", 2_u64))
///     .build();
/// assert_eq!(limits.scale_magnitude_limit().unwrap().maximum(), 2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BigDecimalLimitsBuilder<R, Q = u64>
where
    Q: ResourceQuantity,
{
    /// Limit configuration accumulated by chained builder calls.
    limits: BigDecimalLimits<R, Q>,
}

impl<R, Q> Default for BigDecimalLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an empty builder through the standard [`Default`] interface.
    ///
    /// # Returns
    ///
    /// Creates an empty builder through the standard [`Default`] interface.
    fn default() -> Self {
        Self::new()
    }
}

impl<R, Q> BigDecimalLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an empty decimal-limits builder.
    ///
    /// # Returns
    ///
    /// Creates an empty decimal-limits builder.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            limits: BigDecimalLimits::new(),
        }
    }

    /// Creates a builder retaining an existing limit configuration.
    ///
    /// # Parameters
    ///
    /// * `limits` - Existing decimal limits whose configuration is copied into
    ///   this builder.
    ///
    /// # Returns
    ///
    /// Creates a builder retaining an existing limit configuration.
    #[inline]
    #[must_use]
    pub(crate) const fn from_limits(limits: BigDecimalLimits<R, Q>) -> Self {
        Self { limits }
    }

    /// Sets the coefficient limits.
    ///
    /// # Parameters
    ///
    /// * `limits` - Coefficient limits to apply to the decimal value.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn coefficient_limits(mut self, limits: BigIntegerLimits<R, Q>) -> Self {
        self.limits.set_coefficient_limits(limits);
        self
    }

    /// Sets the absolute scale-magnitude limit.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource-bound absolute scale-magnitude limit to install.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn scale_magnitude_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_scale_magnitude_limit(limit);
        self
    }

    /// Builds the configured decimal limits.
    ///
    /// # Returns
    ///
    /// Builds the configured decimal limits.
    #[inline]
    #[must_use]
    pub fn build(self) -> BigDecimalLimits<R, Q> {
        self.limits
    }
}
