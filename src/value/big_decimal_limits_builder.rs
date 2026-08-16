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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BigDecimalLimitsBuilder<R, Q = u64>
where
    Q: ResourceQuantity,
{
    limits: BigDecimalLimits<R, Q>,
}

impl<R, Q> Default for BigDecimalLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<R, Q> BigDecimalLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an empty decimal-limits builder.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            limits: BigDecimalLimits::new(),
        }
    }

    /// Sets the coefficient limits.
    #[inline]
    #[must_use]
    pub fn coefficient_limits(
        mut self,
        limits: BigIntegerLimits<R, Q>,
    ) -> Self {
        self.limits.set_coefficient_limits(limits);
        self
    }

    /// Sets the absolute scale-magnitude limit.
    #[inline]
    #[must_use]
    pub fn scale_magnitude_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_scale_magnitude_limit(limit);
        self
    }

    /// Builds the configured decimal limits.
    #[inline]
    #[must_use]
    pub fn build(self) -> BigDecimalLimits<R, Q> {
        self.limits
    }
}
