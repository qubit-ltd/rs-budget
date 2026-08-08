// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Mutable accounting state for one resource dimension.

use crate::LimitExceeded;
use crate::ResourceLimit;

/// Mutable consumption state for one resource dimension.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceBudget {
    limit: ResourceLimit,
    used: usize,
}

impl ResourceBudget {
    /// Creates an empty budget for the specified limit.
    ///
    /// # Parameters
    ///
    /// - `limit`: Immutable maximum governing this budget.
    ///
    /// # Returns
    ///
    /// A budget with zero recorded usage.
    #[inline(always)]
    pub const fn new(limit: ResourceLimit) -> Self {
        Self { limit, used: 0 }
    }

    /// Returns the immutable limit governing this budget.
    ///
    /// # Returns
    ///
    /// The configured resource limit.
    #[inline(always)]
    pub const fn limit(self) -> ResourceLimit {
        self.limit
    }

    /// Returns the amount consumed by this budget.
    ///
    /// # Returns
    ///
    /// The cumulative successful consumption.
    #[inline(always)]
    pub const fn used(self) -> usize {
        self.used
    }

    /// Returns the remaining capacity.
    ///
    /// # Returns
    ///
    /// The configured maximum minus successful consumption, saturated at zero.
    #[inline(always)]
    pub const fn remaining(self) -> usize {
        self.limit.maximum().saturating_sub(self.used)
    }

    /// Returns whether the budget has reached its maximum.
    ///
    /// # Returns
    ///
    /// `true` when `used` is at least the configured maximum.
    #[inline(always)]
    pub const fn is_exhausted(self) -> bool {
        self.used >= self.limit.maximum()
    }

    /// Checks additional consumption without changing the budget.
    ///
    /// # Parameters
    ///
    /// - `kind`: Domain-specific resource category to preserve on failure.
    /// - `amount`: Additional amount to check.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the resulting usage remains within the limit.
    ///
    /// # Errors
    ///
    /// Returns [`LimitExceeded`] when the resulting usage exceeds the limit.
    #[inline]
    pub fn check_additional<K>(
        &self,
        kind: K,
        amount: usize,
    ) -> Result<(), LimitExceeded<K>> {
        self.limit.check(kind, self.used.saturating_add(amount))
    }

    /// Consumes an amount when the resulting usage remains within the limit.
    ///
    /// # Parameters
    ///
    /// - `kind`: Domain-specific resource category to preserve on failure.
    /// - `amount`: Additional amount to consume.
    ///
    /// # Returns
    ///
    /// `Ok(())` after recording the consumption.
    ///
    /// # Errors
    ///
    /// Returns [`LimitExceeded`] and leaves the budget unchanged when the
    /// resulting usage exceeds the limit.
    #[inline]
    pub fn consume<K>(
        &mut self,
        kind: K,
        amount: usize,
    ) -> Result<(), LimitExceeded<K>> {
        let observed = self.used.saturating_add(amount);
        self.limit.check(kind, observed)?;
        self.used = observed;
        Ok(())
    }
}
