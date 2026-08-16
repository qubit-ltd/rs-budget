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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BigIntegerLimitsBuilder<R, Q = u64>
where
    Q: ResourceQuantity,
{
    limits: BigIntegerLimits<R, Q>,
}

impl<R, Q> Default for BigIntegerLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<R, Q> BigIntegerLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an empty integer-limits builder.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            limits: BigIntegerLimits::new(),
        }
    }

    /// Sets the magnitude bit-length limit.
    #[inline]
    #[must_use]
    pub fn magnitude_bits_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_magnitude_bits_limit(limit);
        self
    }

    /// Sets the significant decimal digit limit.
    #[inline]
    #[must_use]
    pub fn significant_decimal_digits_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_significant_decimal_digits_limit(limit);
        self
    }

    /// Builds the configured integer limits.
    #[inline]
    #[must_use]
    pub fn build(self) -> BigIntegerLimits<R, Q> {
        self.limits
    }
}
