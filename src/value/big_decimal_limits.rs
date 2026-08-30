// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Bounded coefficient and scale checks for `BigDecimal`.

use bigdecimal::BigDecimal;

use super::BigDecimalLimitsBuilder;
use super::BigIntegerLimits;
use crate::resource::MeasuredBudgetError;
use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;

/// Composes coefficient limits with an absolute scale limit.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Examples
///
/// ```
/// use bigdecimal::BigDecimal;
/// use qubit_budget::BigDecimalLimits;
/// use qubit_budget::ResourceLimit;
///
/// let limits = BigDecimalLimits::builder()
///     .scale_magnitude_limit(ResourceLimit::new("scale", 2_u64))
///     .build();
/// limits.check(&BigDecimal::new(123.into(), 2)).expect("scale two should fit");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BigDecimalLimits<R, Q = u64>
where
    Q: ResourceQuantity,
{
    /// Point limits applied to the arbitrary-precision coefficient.
    coefficient: BigIntegerLimits<R, Q>,
    /// Optional inclusive maximum for the absolute decimal scale.
    max_scale_magnitude: Option<ResourceLimit<R, Q>>,
}

impl<R, Q> BigDecimalLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates limits with no configured decimal bounds.
    ///
    /// # Returns
    ///
    /// Creates limits with no configured decimal bounds.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            coefficient: BigIntegerLimits::new(),
            max_scale_magnitude: None,
        }
    }

    /// Creates a builder for decimal limits.
    ///
    /// # Returns
    ///
    /// Creates a builder for decimal limits.
    #[inline]
    #[must_use]
    pub const fn builder() -> BigDecimalLimitsBuilder<R, Q> {
        BigDecimalLimitsBuilder::new()
    }

    /// Converts these limits into a builder for further configuration.
    ///
    /// # Returns
    ///
    /// Converts these limits into a builder for further configuration.
    #[inline]
    #[must_use]
    pub const fn into_builder(self) -> BigDecimalLimitsBuilder<R, Q> {
        BigDecimalLimitsBuilder::from_limits(self)
    }

    /// Returns the coefficient limits.
    ///
    /// # Returns
    ///
    /// Returns the coefficient limits.
    #[must_use]
    #[inline(always)]
    pub const fn coefficient_limits(&self) -> &BigIntegerLimits<R, Q> {
        &self.coefficient
    }

    /// Returns the configured scale-magnitude limit, if any.
    ///
    /// # Returns
    ///
    /// Returns the configured scale-magnitude limit, if any.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub const fn scale_magnitude_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_scale_magnitude.as_ref()
    }

    /// Checks scale before checking the borrowed coefficient.
    ///
    /// # Parameters
    ///
    /// * `value` - Arbitrary-precision decimal whose scale and coefficient are
    ///   compared with the configured limits.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the operation completes successfully.
    ///
    /// # Errors
    ///
    /// Returns [`MeasuredBudgetError`] when a native measurement cannot fit `Q`
    /// or a configured limit rejects it.
    #[inline]
    pub fn check(&self, value: &BigDecimal) -> Result<(), MeasuredBudgetError<R, Q>>
    where
        R: Clone,
    {
        let (coefficient, scale) = value.as_bigint_and_scale();
        if let Some(limit) = self.max_scale_magnitude.as_ref() {
            let magnitude = Q::try_from_u64(scale.unsigned_abs())
                .map_err(|source| MeasuredBudgetError::quantity(limit.resource().clone(), source))?;
            limit.check(magnitude).map_err(MeasuredBudgetError::from)?;
        }
        self.coefficient.check(coefficient.as_ref())
    }

    /// Replaces coefficient limits during builder composition.
    ///
    /// # Parameters
    ///
    /// * `limits` - Coefficient limits to apply to the decimal value.
    #[inline(always)]
    pub(super) fn set_coefficient_limits(&mut self, limits: BigIntegerLimits<R, Q>) {
        self.coefficient = limits;
    }

    /// Replaces the scale-magnitude limit during builder composition.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource-bound absolute scale-magnitude limit to install.
    #[inline(always)]
    pub(super) fn set_scale_magnitude_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.max_scale_magnitude = Some(limit);
    }
}

impl<R, Q> Default for BigDecimalLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates unconfigured decimal limits.
    ///
    /// # Returns
    ///
    /// Creates unconfigured decimal limits.
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
