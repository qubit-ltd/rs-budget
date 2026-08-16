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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StringLimits<R, Q = u64>
where
    Q: ResourceQuantity,
{
    max_utf8_bytes: Option<ResourceLimit<R, Q>>,
}

impl<R, Q> StringLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates limits with no configured string bound.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_utf8_bytes: None,
        }
    }

    /// Creates a builder for string limits.
    #[inline]
    #[must_use]
    pub const fn builder() -> StringLimitsBuilder<R, Q> {
        StringLimitsBuilder::new()
    }

    /// Returns the configured UTF-8 byte limit, if any.
    #[must_use]
    #[inline(always)]
    pub const fn utf8_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_utf8_bytes.as_ref()
    }

    /// Checks one string without mutating the limits.
    ///
    /// The measured quantity is the string's UTF-8 byte length. A configured
    /// limit returns a point budget error when the length is too large.
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
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
