// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Bounded magnitude and significant decimal digit checks for `BigInt`.

use num_bigint::BigInt;

use super::BigIntegerLimitsBuilder;
use crate::resource::LimitExceededError;
use crate::resource::MeasuredBudgetError;
use crate::resource::Observation;
use crate::resource::QuantityConversionError;
use crate::resource::QuantityMeasurement;
use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;
use crate::resource::quantity_as_u128;

/// Optional point limits for one arbitrary-precision integer.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Examples
///
/// ```
/// use num_bigint::BigInt;
/// use qubit_budget::BigIntegerLimits;
/// use qubit_budget::ResourceLimit;
///
/// let limits = BigIntegerLimits::builder()
///     .magnitude_bits_limit(ResourceLimit::new("integer bits", 8_u64))
///     .build();
/// limits.check(&BigInt::from(255_u16)).expect("255 needs eight bits");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BigIntegerLimits<R, Q = u64>
where
    Q: ResourceQuantity,
{
    /// Optional inclusive maximum for the unsigned magnitude bit length.
    max_magnitude_bits: Option<ResourceLimit<R, Q>>,
    /// Optional inclusive maximum for significant decimal digits.
    max_significant_decimal_digits: Option<ResourceLimit<R, Q>>,
}

impl<R, Q> BigIntegerLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates limits with no configured integer bounds.
    ///
    /// # Returns
    ///
    /// The returned value has neither magnitude nor decimal-digit bounds
    /// configured.
    #[inline]
    #[must_use = "the integer limits should be used"]
    pub const fn new() -> Self {
        Self {
            max_magnitude_bits: None,
            max_significant_decimal_digits: None,
        }
    }

    /// Creates a builder for integer limits.
    ///
    /// # Returns
    ///
    /// The returned builder starts without configured integer bounds.
    #[inline]
    #[must_use]
    pub const fn builder() -> BigIntegerLimitsBuilder<R, Q> {
        BigIntegerLimitsBuilder::new()
    }

    /// Converts these limits into a builder for further configuration.
    ///
    /// # Returns
    ///
    /// The returned builder starts with this value's existing configuration.
    #[inline]
    #[must_use]
    pub const fn into_builder(self) -> BigIntegerLimitsBuilder<R, Q> {
        BigIntegerLimitsBuilder::from_limits(self)
    }

    /// Returns the configured magnitude bit-length limit, if any.
    ///
    /// # Returns
    ///
    /// Returns the configured magnitude bit-length limit, if any.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[inline]
    #[must_use]
    pub const fn magnitude_bits_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_magnitude_bits.as_ref()
    }

    /// Returns the configured significant decimal digit limit, if any.
    ///
    /// # Returns
    ///
    /// Returns the configured significant decimal digit limit, if any.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[inline]
    #[must_use]
    pub const fn significant_decimal_digits_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_significant_decimal_digits.as_ref()
    }

    /// Checks one integer without formatting clearly oversized values.
    ///
    /// Values near the decimal boundary are formatted once. Clearly oversized
    /// values report a conservative lower bound instead of allocating a
    /// decimal string proportional to the input magnitude.
    ///
    /// # Parameters
    ///
    /// * `value` - Arbitrary-precision integer whose magnitude and decimal
    ///   digit counts are compared with the configured limits.
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
    pub fn check(&self, value: &BigInt) -> Result<(), MeasuredBudgetError<R, Q>>
    where
        R: Clone,
    {
        self.check_measurements(value.bits(), || {
            let text = value.to_str_radix(10);
            text.strip_prefix('-').unwrap_or(&text).len()
        })
    }

    /// Checks precomputed integer measurements against the configured limits.
    ///
    /// This permits values supplied by compatible arbitrary-precision integer
    /// crates to share the same limit-checking logic without conversion. The
    /// decimal digit callback is evaluated only when the bit-length estimate
    /// cannot determine the result.
    ///
    /// # Parameters
    ///
    /// * `magnitude_bits` - Unsigned bit length of the integer magnitude.
    /// * `decimal_digits` - Supplies the exact significant decimal digit count.
    ///
    /// # Returns
    ///
    /// `Ok(())` when both measurements satisfy their configured limits.
    ///
    /// # Errors
    ///
    /// Returns [`MeasuredBudgetError`] when a measurement cannot fit `Q` or a
    /// configured limit rejects it.
    #[inline]
    pub(super) fn check_measurements<F>(
        &self,
        magnitude_bits: u64,
        decimal_digits: F,
    ) -> Result<(), MeasuredBudgetError<R, Q>>
    where
        R: Clone,
        F: FnOnce() -> usize,
    {
        if let Some(limit) = self.max_magnitude_bits.as_ref() {
            let bits = Q::try_from_u64(magnitude_bits)
                .map_err(|source| MeasuredBudgetError::quantity(limit.resource().clone(), source))?;
            limit.check(bits).map_err(MeasuredBudgetError::from)?;
        }
        if let Some(limit) = self.max_significant_decimal_digits.as_ref() {
            check_decimal_digits(limit, magnitude_bits, decimal_digits)?;
        }
        Ok(())
    }

    /// Replaces the magnitude-bit limit during builder composition.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource-bound magnitude bit-length limit to install.
    pub(super) fn set_magnitude_bits_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.max_magnitude_bits = Some(limit);
    }

    /// Replaces the decimal-digit limit during builder composition.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource-bound significant decimal-digit limit to install.
    pub(super) fn set_significant_decimal_digits_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.max_significant_decimal_digits = Some(limit);
    }
}

impl<R, Q> Default for BigIntegerLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates unconfigured integer limits.
    ///
    /// # Returns
    ///
    /// Creates unconfigured integer limits.
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Counts and validates significant decimal digits, formatting only near the
/// configured boundary.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Parameters
///
/// * `limit` - Resource-bound inclusive decimal-digit maximum.
/// * `magnitude_bits` - Unsigned bit length of the integer magnitude.
/// * `decimal_digits` - Lazily supplies the exact significant decimal digit
///   count when the bit-length estimate is inconclusive.
///
/// # Returns
///
/// `Ok(())` when the significant digit count fits the configured maximum.
///
/// # Errors
///
/// Returns [`MeasuredBudgetError`] when the exact digit count cannot fit `Q`
/// or exceeds `limit`.
fn check_decimal_digits<R, Q, F>(
    limit: &ResourceLimit<R, Q>,
    magnitude_bits: u64,
    decimal_digits: F,
) -> Result<(), MeasuredBudgetError<R, Q>>
where
    R: Clone,
    Q: ResourceQuantity,
    F: FnOnce() -> usize,
{
    if magnitude_bits == 0 {
        return Ok(());
    }

    let maximum = limit.maximum();
    let wide_maximum = quantity_as_u128(maximum);
    let upper = magnitude_bits.div_ceil(3);
    if u128::from(upper) <= wide_maximum {
        return Ok(());
    }
    let lower = (magnitude_bits - 1) / 4 + 1;
    if u128::from(lower) > wide_maximum {
        return match Q::try_from_u64(lower) {
            Ok(observed) => Err(LimitExceededError::at_least(limit.resource().clone(), observed, maximum).into()),
            Err(source) => Err(MeasuredBudgetError::quantity(
                limit.resource().clone(),
                QuantityConversionError::new(QuantityMeasurement::AtLeastU64(lower), source.target()),
            )),
        };
    }

    let digits = Q::try_from_usize(decimal_digits())
        .map_err(|source| MeasuredBudgetError::quantity(limit.resource().clone(), source))?;
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

#[cfg(test)]
mod tests {
    use super::check_decimal_digits;
    use crate::resource::ResourceLimit;

    #[test]
    fn test_digit_estimates_skip_formatting_when_decisive() {
        let fits = ResourceLimit::new("digits", 100_u8);
        check_decimal_digits(&fits, 257, || panic!("must not format")).expect("upper estimate is within the maximum");

        let exceeds = ResourceLimit::new("digits", 255_u8);
        let error = check_decimal_digits(&exceeds, 16_385, || panic!("must not format"))
            .expect_err("lower estimate exceeds the maximum");
        assert!(error.quantity_error().is_some());
    }
}
