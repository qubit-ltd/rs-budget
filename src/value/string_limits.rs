// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! UTF-8 byte limits for one string value.

use crate::BudgetError;
use crate::ResourceLimit;

/// Optional point limit for one UTF-8 string's byte length.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StringLimits<R> {
    max_utf8_bytes: Option<ResourceLimit<R, u64>>,
}

impl<R> StringLimits<R> {
    /// Creates limits with no configured string bound.
    #[inline]
    pub const fn empty() -> Self {
        Self {
            max_utf8_bytes: None,
        }
    }

    /// Adds an inclusive UTF-8 byte limit.
    #[inline]
    pub fn with_utf8_bytes_limit(
        mut self,
        limit: ResourceLimit<R, u64>,
    ) -> Self {
        self.max_utf8_bytes = Some(limit);
        self
    }

    /// Returns the configured UTF-8 byte limit, if any.
    #[inline(always)]
    pub const fn utf8_bytes_limit(&self) -> Option<&ResourceLimit<R, u64>> {
        self.max_utf8_bytes.as_ref()
    }

    /// Checks one string without mutating the limits.
    ///
    /// The measured quantity is the string's UTF-8 byte length. A configured
    /// limit returns a point `BudgetError` when the length is too large.
    #[inline]
    pub fn check(&self, value: &str) -> Result<(), BudgetError<R, u64>>
    where
        R: Clone,
    {
        let bytes = u64::try_from(value.len()).expect("Rust usize fits in u64");
        match self.max_utf8_bytes.as_ref() {
            Some(limit) => limit.check(bytes),
            None => Ok(()),
        }
    }
}

impl<R> Default for StringLimits<R> {
    /// Creates unconfigured string limits.
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}
