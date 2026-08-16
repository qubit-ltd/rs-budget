// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow multiple-public-types
//! Bounded magnitude and significant decimal digit checks for `BigInt`.

use num_bigint::BigInt;

use crate::LimitExceededError;
use crate::MeasuredBudgetError;
use crate::Observation;
use crate::ResourceLimit;
use crate::ResourceQuantity;

/// Optional point limits for one arbitrary-precision integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BigIntegerLimits<R, Q = u64>
where
    Q: ResourceQuantity,
{
    max_magnitude_bits: Option<ResourceLimit<R, Q>>,
    max_significant_decimal_digits: Option<ResourceLimit<R, Q>>,
}

impl<R, Q> BigIntegerLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates limits with no configured integer bounds.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_magnitude_bits: None,
            max_significant_decimal_digits: None,
        }
    }

    /// Creates a builder for integer limits.
    #[inline]
    #[must_use]
    pub const fn builder() -> BigIntegerLimitsBuilder<R, Q> {
        BigIntegerLimitsBuilder::new()
    }

    /// Returns the configured magnitude bit-length limit, if any.
    #[must_use]
    #[inline(always)]
    pub const fn magnitude_bits_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_magnitude_bits.as_ref()
    }

    /// Returns the configured significant decimal digit limit, if any.
    #[must_use]
    #[inline(always)]
    pub const fn significant_decimal_digits_limit(
        &self,
    ) -> Option<&ResourceLimit<R, Q>> {
        self.max_significant_decimal_digits.as_ref()
    }

    /// Checks one integer without formatting clearly oversized values.
    ///
    /// Values near the decimal boundary are formatted once. Clearly oversized
    /// values report a conservative lower bound instead of allocating a
    /// decimal string proportional to the input magnitude.
    #[inline]
    pub fn check(&self, value: &BigInt) -> Result<(), MeasuredBudgetError<R, Q>>
    where
        R: Clone,
    {
        if let Some(limit) = self.max_magnitude_bits.as_ref() {
            let bits = Q::try_from_u64(value.bits()).map_err(|source| {
                MeasuredBudgetError::quantity(limit.resource().clone(), source)
            })?;
            limit.check(bits).map_err(MeasuredBudgetError::from)?;
        }
        if let Some(limit) = self.max_significant_decimal_digits.as_ref() {
            check_decimal_digits(limit, value)?;
        }
        Ok(())
    }
}

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
        self.limits.max_magnitude_bits = Some(limit);
        self
    }

    /// Sets the significant decimal digit limit.
    #[inline]
    #[must_use]
    pub fn significant_decimal_digits_limit(
        mut self,
        limit: ResourceLimit<R, Q>,
    ) -> Self {
        self.limits.max_significant_decimal_digits = Some(limit);
        self
    }

    /// Builds the configured integer limits.
    #[inline]
    #[must_use]
    pub fn build(self) -> BigIntegerLimits<R, Q> {
        self.limits
    }
}

impl<R, Q> Default for BigIntegerLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates unconfigured integer limits.
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

fn check_decimal_digits<R, Q>(
    limit: &ResourceLimit<R, Q>,
    value: &BigInt,
) -> Result<(), MeasuredBudgetError<R, Q>>
where
    R: Clone,
    Q: ResourceQuantity,
{
    let bits = value.bits();
    if bits == 0 {
        return Ok(());
    }

    let maximum = limit.maximum();
    let bits = Q::try_from_u64(bits).map_err(|source| {
        MeasuredBudgetError::quantity(limit.resource().clone(), source)
    })?;
    let low_bits = maximum
        .checked_add(maximum)
        .and_then(|value| value.checked_add(maximum));
    if low_bits.is_some_and(|low_bits| bits <= low_bits) {
        return Ok(());
    }
    let high_bits = low_bits.and_then(|value| value.checked_add(maximum));
    if high_bits.is_some_and(|high_bits| bits > high_bits) {
        let Some(observed) = maximum.checked_add(Q::ONE) else {
            return Ok(());
        };
        return Err(LimitExceededError {
            resource: limit.resource().clone(),
            observed: Observation::AtLeast(observed),
            maximum,
        }
        .into());
    }

    let text = value.to_str_radix(10);
    let digits = text.strip_prefix('-').unwrap_or(&text).len();
    let digits = Q::try_from_usize(digits).map_err(|source| {
        MeasuredBudgetError::quantity(limit.resource().clone(), source)
    })?;
    if digits > maximum {
        Err(LimitExceededError {
            resource: limit.resource().clone(),
            observed: Observation::Exact(digits),
            maximum,
        }
        .into())
    } else {
        Ok(())
    }
}
