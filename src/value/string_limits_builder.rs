// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Builds UTF-8 byte limits for one string value.

use super::StringLimits;
use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;

/// Builder for [`StringLimits`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StringLimitsBuilder<R, Q = u64>
where
    Q: ResourceQuantity,
{
    limits: StringLimits<R, Q>,
}

impl<R, Q> Default for StringLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<R, Q> StringLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an empty string-limits builder.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            limits: StringLimits::new(),
        }
    }

    /// Creates a builder retaining an existing limit configuration.
    #[inline]
    #[must_use]
    pub(crate) const fn from_limits(limits: StringLimits<R, Q>) -> Self {
        Self { limits }
    }

    /// Sets the inclusive UTF-8 byte limit.
    #[inline]
    #[must_use]
    pub fn utf8_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_utf8_bytes_limit(limit);
        self
    }

    /// Builds the configured string limits.
    #[inline]
    #[must_use]
    pub fn build(self) -> StringLimits<R, Q> {
        self.limits
    }
}
