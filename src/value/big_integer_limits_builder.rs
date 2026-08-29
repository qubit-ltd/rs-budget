// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Builds bounded magnitude and decimal digit limits for `BigInt`.

use super::BigIntegerLimits;
use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;

/// Builder for [`BigIntegerLimits`].
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Examples
///
/// ```
/// use qubit_budget::BigIntegerLimitsBuilder;
/// use qubit_budget::ResourceLimit;
///
/// let limits = BigIntegerLimitsBuilder::new()
///     .significant_decimal_digits_limit(ResourceLimit::new("digits", 3_u64))
///     .build();
/// assert_eq!(limits.significant_decimal_digits_limit().unwrap().maximum(), 3);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BigIntegerLimitsBuilder<R, Q = u64>
where
    Q: ResourceQuantity,
{
    /// Limit configuration accumulated by chained builder calls.
    limits: BigIntegerLimits<R, Q>,
}

impl<R, Q> Default for BigIntegerLimitsBuilder<R, Q>
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

impl<R, Q> BigIntegerLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an empty integer-limits builder.
    ///
    /// # Returns
    ///
    /// Creates an empty integer-limits builder.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            limits: BigIntegerLimits::new(),
        }
    }

    /// Creates a builder retaining an existing limit configuration.
    ///
    /// # Parameters
    ///
    /// * `limits` - Immutable limit configuration used by the operation.
    ///
    /// # Returns
    ///
    /// Creates a builder retaining an existing limit configuration.
    #[inline]
    #[must_use]
    pub(crate) const fn from_limits(limits: BigIntegerLimits<R, Q>) -> Self {
        Self { limits }
    }

    /// Sets the magnitude bit-length limit.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource-bound limit to inspect or install.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn magnitude_bits_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_magnitude_bits_limit(limit);
        self
    }

    /// Sets the significant decimal digit limit.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource-bound limit to inspect or install.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn significant_decimal_digits_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_significant_decimal_digits_limit(limit);
        self
    }

    /// Builds the configured integer limits.
    ///
    /// # Returns
    ///
    /// Builds the configured integer limits.
    #[inline]
    #[must_use]
    pub fn build(self) -> BigIntegerLimits<R, Q> {
        self.limits
    }
}
