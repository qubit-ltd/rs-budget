// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! UTF-8 byte limits for one string value.

use super::StringLimitsBuilder;
use crate::resource::MeasuredBudgetError;
use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;

/// Optional point limit for one UTF-8 string's byte length.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Examples
///
/// ```
/// use qubit_budget::ResourceLimit;
/// use qubit_budget::StringLimits;
///
/// let limits = StringLimits::builder()
///     .utf8_bytes_limit(ResourceLimit::new("name bytes", 5_u64))
///     .build();
/// limits.check("hello").expect("five UTF-8 bytes should fit");
/// assert!(limits.check("hello!").is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StringLimits<R, Q = u64>
where
    Q: ResourceQuantity,
{
    /// Optional inclusive maximum for one string's UTF-8 byte length.
    max_utf8_bytes: Option<ResourceLimit<R, Q>>,
}

impl<R, Q> StringLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates limits with no configured string bound.
    ///
    /// # Returns
    ///
    /// Creates limits with no configured string bound.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { max_utf8_bytes: None }
    }

    /// Creates a builder for string limits.
    ///
    /// # Returns
    ///
    /// Creates a builder for string limits.
    #[inline]
    #[must_use]
    pub const fn builder() -> StringLimitsBuilder<R, Q> {
        StringLimitsBuilder::new()
    }

    /// Converts these limits into a builder for further configuration.
    ///
    /// # Returns
    ///
    /// Converts these limits into a builder for further configuration.
    #[inline]
    #[must_use]
    pub const fn into_builder(self) -> StringLimitsBuilder<R, Q> {
        StringLimitsBuilder::from_limits(self)
    }

    /// Returns the configured UTF-8 byte limit, if any.
    ///
    /// # Returns
    ///
    /// Returns the configured UTF-8 byte limit, if any.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub const fn utf8_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_utf8_bytes.as_ref()
    }

    /// Checks one string without mutating the limits.
    ///
    /// The measured quantity is the string's UTF-8 byte length. A configured
    /// limit returns a point budget error when the length is too large.
    ///
    /// # Parameters
    ///
    /// * `value` - UTF-8 string whose byte length is compared with the limit.
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
    pub fn check(&self, value: &str) -> Result<(), MeasuredBudgetError<R, Q>>
    where
        R: Clone,
    {
        let Some(limit) = self.max_utf8_bytes.as_ref() else {
            return Ok(());
        };
        let bytes = Q::try_from_usize(value.len())
            .map_err(|source| MeasuredBudgetError::quantity(limit.resource().clone(), source))?;
        limit.check(bytes).map_err(MeasuredBudgetError::from)
    }

    /// Replaces the UTF-8 byte limit during builder composition.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource-bound UTF-8 byte limit to install.
    #[inline(always)]
    pub(super) fn set_utf8_bytes_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.max_utf8_bytes = Some(limit);
    }
}

impl<R, Q> Default for StringLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates unconfigured string limits.
    ///
    /// # Returns
    ///
    /// Creates unconfigured string limits.
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
