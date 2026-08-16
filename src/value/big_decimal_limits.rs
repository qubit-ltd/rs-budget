// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Bounded coefficient and scale checks for `BigDecimal`.

use bigdecimal::BigDecimal;

use super::BigIntegerLimits;
use crate::MeasuredBudgetError;
use crate::ResourceLimit;
use crate::ResourceQuantity;

/// Composes coefficient limits with an absolute scale limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BigDecimalLimits<R, Q = u64>
where
    Q: ResourceQuantity,
{
    coefficient: BigIntegerLimits<R, Q>,
    max_scale_magnitude: Option<ResourceLimit<R, Q>>,
}

impl<R, Q> BigDecimalLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates limits with no configured decimal bounds.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            coefficient: BigIntegerLimits::new(),
            max_scale_magnitude: None,
        }
    }

    /// Replaces the coefficient limits.
    #[inline]
    #[must_use]
    pub fn with_coefficient_limits(
        mut self,
        limits: BigIntegerLimits<R, Q>,
    ) -> Self {
        self.coefficient = limits;
        self
    }

    /// Adds an inclusive absolute scale-magnitude limit.
    #[inline]
    #[must_use]
    pub fn with_scale_magnitude_limit(
        mut self,
        limit: ResourceLimit<R, Q>,
    ) -> Self {
        self.max_scale_magnitude = Some(limit);
        self
    }

    /// Returns the coefficient limits.
    #[must_use]
    #[inline(always)]
    pub const fn coefficient_limits(&self) -> &BigIntegerLimits<R, Q> {
        &self.coefficient
    }

    /// Returns the configured scale-magnitude limit, if any.
    #[must_use]
    #[inline(always)]
    pub const fn scale_magnitude_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_scale_magnitude.as_ref()
    }

    /// Checks scale before checking the borrowed coefficient.
    #[inline]
    pub fn check(
        &self,
        value: &BigDecimal,
    ) -> Result<(), MeasuredBudgetError<R, Q>>
    where
        R: Clone,
    {
        let (coefficient, scale) = value.as_bigint_and_scale();
        if let Some(limit) = self.max_scale_magnitude.as_ref() {
            let magnitude =
                Q::try_from_u64(scale.unsigned_abs()).map_err(|source| {
                    MeasuredBudgetError::quantity(
                        limit.resource().clone(),
                        source,
                    )
                })?;
            limit.check(magnitude).map_err(MeasuredBudgetError::from)?;
        }
        self.coefficient.check(coefficient.as_ref())
    }
}

impl<R, Q> Default for BigDecimalLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates unconfigured decimal limits.
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
