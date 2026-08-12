// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! Bounded magnitude and significant decimal digit checks for `BigInt`.

use num_bigint::BigInt;

use crate::BudgetError;
use crate::Observation;
use crate::ResourceLimit;

/// Optional point limits for one arbitrary-precision integer.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BigIntegerLimits<R> {
    max_magnitude_bits: Option<ResourceLimit<R, u64>>,
    max_significant_decimal_digits: Option<ResourceLimit<R, u64>>,
}

impl<R> BigIntegerLimits<R> {
    /// Creates limits with no configured integer bounds.
    #[inline]
    pub const fn empty() -> Self {
        Self {
            max_magnitude_bits: None,
            max_significant_decimal_digits: None,
        }
    }

    /// Adds an inclusive magnitude bit-length limit.
    #[inline]
    pub fn with_magnitude_bits_limit(
        mut self,
        limit: ResourceLimit<R, u64>,
    ) -> Self {
        self.max_magnitude_bits = Some(limit);
        self
    }

    /// Adds an inclusive significant decimal digit limit.
    #[inline]
    pub fn with_significant_decimal_digits_limit(
        mut self,
        limit: ResourceLimit<R, u64>,
    ) -> Self {
        self.max_significant_decimal_digits = Some(limit);
        self
    }

    /// Returns the configured magnitude bit-length limit, if any.
    #[inline(always)]
    pub const fn magnitude_bits_limit(&self) -> Option<&ResourceLimit<R, u64>> {
        self.max_magnitude_bits.as_ref()
    }

    /// Returns the configured significant decimal digit limit, if any.
    #[inline(always)]
    pub const fn significant_decimal_digits_limit(
        &self,
    ) -> Option<&ResourceLimit<R, u64>> {
        self.max_significant_decimal_digits.as_ref()
    }

    /// Checks one integer without formatting clearly oversized values.
    ///
    /// Values near the decimal boundary are formatted once. Clearly oversized
    /// values report a conservative lower bound instead of allocating a
    /// decimal string proportional to the input magnitude.
    #[inline]
    pub fn check(&self, value: &BigInt) -> Result<(), BudgetError<R, u64>>
    where
        R: Clone,
    {
        if let Some(limit) = self.max_magnitude_bits.as_ref() {
            limit.check(value.bits())?;
        }
        if let Some(limit) = self.max_significant_decimal_digits.as_ref() {
            check_decimal_digits(limit, value)?;
        }
        Ok(())
    }
}

impl<R> Default for BigIntegerLimits<R> {
    /// Creates unconfigured integer limits.
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

fn check_decimal_digits<R>(
    limit: &ResourceLimit<R, u64>,
    value: &BigInt,
) -> Result<(), BudgetError<R, u64>>
where
    R: Clone,
{
    let bits = value.bits();
    if bits == 0 {
        return Ok(());
    }

    let maximum = limit.maximum();
    let low_bits = maximum.saturating_mul(3);
    let high_bits = maximum.saturating_mul(4);
    if bits <= low_bits || maximum == u64::MAX {
        return Ok(());
    }
    if bits > high_bits {
        return Err(BudgetError::LimitExceeded {
            resource: limit.resource().clone(),
            observed: Observation::AtLeast(maximum.saturating_add(1)),
            maximum,
        });
    }

    let text = value.to_str_radix(10);
    let digits = text.strip_prefix('-').unwrap_or(&text).len();
    let digits = u64::try_from(digits).expect("Rust usize fits in u64");
    if digits > maximum {
        Err(BudgetError::LimitExceeded {
            resource: limit.resource().clone(),
            observed: Observation::Exact(digits),
            maximum,
        })
    } else {
        Ok(())
    }
}
