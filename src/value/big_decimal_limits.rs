// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! Bounded coefficient and scale checks for `BigDecimal`.

use bigdecimal::BigDecimal;

use super::BigIntegerLimits;
use crate::BudgetError;
use crate::ResourceLimit;

/// Composes coefficient limits with an absolute scale limit.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BigDecimalLimits<R> {
    coefficient: BigIntegerLimits<R>,
    max_scale_magnitude: Option<ResourceLimit<R, u64>>,
}

impl<R> BigDecimalLimits<R> {
    /// Creates limits with no configured decimal bounds.
    #[inline]
    pub const fn empty() -> Self {
        Self {
            coefficient: BigIntegerLimits::empty(),
            max_scale_magnitude: None,
        }
    }

    /// Replaces the coefficient limits.
    #[inline]
    pub fn with_coefficient_limits(
        mut self,
        limits: BigIntegerLimits<R>,
    ) -> Self {
        self.coefficient = limits;
        self
    }

    /// Adds an inclusive absolute scale-magnitude limit.
    #[inline]
    pub fn with_scale_magnitude_limit(
        mut self,
        limit: ResourceLimit<R, u64>,
    ) -> Self {
        self.max_scale_magnitude = Some(limit);
        self
    }

    /// Returns the coefficient limits.
    #[inline(always)]
    pub const fn coefficient_limits(&self) -> &BigIntegerLimits<R> {
        &self.coefficient
    }

    /// Returns the configured scale-magnitude limit, if any.
    #[inline(always)]
    pub const fn scale_magnitude_limit(
        &self,
    ) -> Option<&ResourceLimit<R, u64>> {
        self.max_scale_magnitude.as_ref()
    }

    /// Checks scale before checking the borrowed coefficient.
    #[inline]
    pub fn check(&self, value: &BigDecimal) -> Result<(), BudgetError<R, u64>>
    where
        R: Clone,
    {
        let (coefficient, scale) = value.as_bigint_and_scale();
        if let Some(limit) = self.max_scale_magnitude.as_ref() {
            limit.check(scale.unsigned_abs())?;
        }
        self.coefficient.check(coefficient.as_ref())
    }
}

impl<R> Default for BigDecimalLimits<R> {
    /// Creates unconfigured decimal limits.
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}
