// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Immutable limits for one resource dimension.

use crate::LimitExceeded;

/// Immutable maximum for one resource dimension.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceLimit {
    maximum: usize,
}

impl ResourceLimit {
    /// The largest representable resource limit.
    pub const UNBOUNDED: Self = Self::new(usize::MAX);

    /// Creates a resource limit with the specified maximum.
    ///
    /// # Parameters
    ///
    /// - `maximum`: Largest permitted resource value.
    ///
    /// # Returns
    ///
    /// A resource limit accepting values through `maximum`.
    #[inline(always)]
    pub const fn new(maximum: usize) -> Self {
        Self { maximum }
    }

    /// Returns the largest representable resource limit.
    ///
    /// # Returns
    ///
    /// A limit whose maximum is `usize::MAX`.
    #[inline(always)]
    pub const fn unbounded() -> Self {
        Self::UNBOUNDED
    }

    /// Returns the configured maximum.
    ///
    /// # Returns
    ///
    /// The largest permitted resource value.
    #[inline(always)]
    pub const fn maximum(self) -> usize {
        self.maximum
    }

    /// Returns whether this limit uses `usize::MAX`.
    ///
    /// # Returns
    ///
    /// `true` when the limit uses the unbounded representation.
    #[inline(always)]
    pub const fn is_unbounded(self) -> bool {
        self.maximum == usize::MAX
    }

    /// Checks an observed value against this limit.
    ///
    /// # Parameters
    ///
    /// - `kind`: Domain-specific resource category to preserve on failure.
    /// - `observed`: Value observed by the caller.
    ///
    /// # Returns
    ///
    /// `Ok(())` when `observed` is within the limit.
    ///
    /// # Errors
    ///
    /// Returns [`LimitExceeded`] when `observed` is greater than the maximum.
    #[inline]
    pub fn check<K>(
        self,
        kind: K,
        observed: usize,
    ) -> Result<(), LimitExceeded<K>> {
        if observed > self.maximum {
            Err(LimitExceeded::new(kind, self.maximum, observed))
        } else {
            Ok(())
        }
    }

    /// Creates an empty budget governed by this limit.
    ///
    /// # Returns
    ///
    /// A mutable budget with zero recorded usage.
    #[inline(always)]
    pub const fn budget(self) -> crate::ResourceBudget {
        crate::ResourceBudget::new(self)
    }
}
