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
    /// Creates a builder with neither coefficient nor scale limits configured.
    ///
    /// # Returns
    ///
    /// A builder equivalent to [`BigDecimalLimitsBuilder::new`].
    fn default() -> Self {
        Self::new()
    }
}

impl<R, Q> BigDecimalLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates a decimal-limits builder with no coefficient or scale limit.
    ///
    /// # Returns
    ///
    /// A builder ready to receive coefficient and scale settings.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            limits: BigDecimalLimits::new(),
        }
    }

    /// Creates a builder initialized from an existing limit configuration.
    ///
    /// # Parameters
    ///
    /// * `limits` - Existing decimal limits to use as the initial
    ///   configuration.
    ///
    /// # Returns
    ///
    /// A builder containing the supplied coefficient and scale settings.
    #[inline]
    #[must_use]
    pub(crate) const fn from_limits(limits: BigDecimalLimits<R, Q>) -> Self {
        Self { limits }
    }

    /// Replaces the coefficient limits while preserving the scale setting.
    ///
    /// # Parameters
    ///
    /// * `limits` - Coefficient limits to use in the resulting configuration.
    ///
    /// # Returns
    ///
    /// A builder whose coefficient limit is set to `limits`.
    #[inline]
    #[must_use]
    pub fn coefficient_limits(mut self, limits: BigIntegerLimits<R, Q>) -> Self {
        self.limits.set_coefficient_limits(limits);
        self
    }

    /// Replaces the absolute scale-magnitude limit while preserving coefficient
    /// limits.
    ///
    /// # Parameters
    ///
    /// * `limit` - Absolute scale-magnitude limit to use in the resulting
    ///   configuration.
    ///
    /// # Returns
    ///
    /// A builder whose scale-magnitude limit is set to `limit`.
    #[inline]
    #[must_use]
    pub fn scale_magnitude_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_scale_magnitude_limit(limit);
        self
    }

    /// Consumes the builder and returns its accumulated decimal-limit settings.
    ///
    /// # Returns
    ///
    /// The configured limits, with no additional validation or allocation.
    #[inline]
    #[must_use]
    pub fn build(self) -> BigDecimalLimits<R, Q> {
        self.limits
    }
}
