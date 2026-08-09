// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Mutable accounting state for one resource dimension.

use crate::InvalidRelease;
use crate::LimitExceeded;
use crate::ResourceLimit;

/// Mutable consumption state for one resource dimension.
#[must_use]
#[derive(Debug, PartialEq, Eq)]
pub struct ResourceBudget {
    limit: ResourceLimit,
    remaining: usize,
}

impl ResourceBudget {
    /// Creates an unused budget for the specified limit.
    ///
    /// # Parameters
    ///
    /// - `limit`: Immutable maximum governing this budget.
    ///
    /// # Returns
    ///
    /// A budget with all capacity available.
    #[inline(always)]
    pub const fn new(limit: ResourceLimit) -> Self {
        Self {
            remaining: limit.maximum(),
            limit,
        }
    }

    /// Returns the immutable limit governing this budget.
    ///
    /// # Returns
    ///
    /// The configured resource limit.
    #[inline(always)]
    pub const fn limit(&self) -> ResourceLimit {
        self.limit
    }

    /// Returns the configured maximum.
    ///
    /// # Returns
    ///
    /// The largest amount this budget can hold or consume.
    #[inline(always)]
    pub const fn maximum(&self) -> usize {
        self.limit.maximum()
    }

    /// Returns the remaining capacity.
    ///
    /// # Returns
    ///
    /// The capacity not yet consumed by this budget.
    #[inline(always)]
    pub const fn remaining(&self) -> usize {
        self.remaining
    }

    /// Returns the amount consumed by this budget.
    ///
    /// # Returns
    ///
    /// The configured maximum minus the remaining capacity.
    #[inline(always)]
    pub const fn used(&self) -> usize {
        self.maximum() - self.remaining
    }

    /// Returns whether no capacity remains.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.remaining == 0
    }

    /// Returns whether this budget has not consumed any capacity.
    #[inline(always)]
    pub const fn is_unused(&self) -> bool {
        self.remaining == self.maximum()
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
        if amount <= self.remaining {
            Ok(())
        } else {
            Err(LimitExceeded::new(
                kind,
                self.maximum(),
                self.used().saturating_add(amount),
            ))
        }
    }

    /// Consumes an amount while preserving the budget when it does not fit.
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
    pub fn try_consume<K>(
        &mut self,
        kind: K,
        amount: usize,
    ) -> Result<(), LimitExceeded<K>> {
        self.check_additional(kind, amount)?;
        self.remaining -= amount;
        Ok(())
    }

    /// Consumes an amount and exhausts the remaining capacity when it does not
    /// fit.
    ///
    /// # Parameters
    ///
    /// - `kind`: Domain-specific resource category to preserve on failure.
    /// - `amount`: Additional amount to consume.
    ///
    /// # Returns
    ///
    /// `Ok(())` after recording the consumption when `amount` fits.
    ///
    /// # Errors
    ///
    /// Returns [`LimitExceeded`] and sets the remaining capacity to zero when
    /// `amount` exceeds the available capacity.
    #[inline]
    pub fn consume_or_exhaust<K>(
        &mut self,
        kind: K,
        amount: usize,
    ) -> Result<(), LimitExceeded<K>> {
        if amount <= self.remaining {
            self.remaining -= amount;
            Ok(())
        } else {
            let error = LimitExceeded::new(
                kind,
                self.maximum(),
                self.used().saturating_add(amount),
            );
            self.remaining = 0;
            Err(error)
        }
    }

    /// Consumes as much of the requested amount as remains available.
    ///
    /// # Parameters
    ///
    /// - `amount`: Maximum additional amount to consume.
    ///
    /// # Returns
    ///
    /// The amount actually consumed, which is at most `amount`.
    #[inline]
    pub fn consume_available(&mut self, amount: usize) -> usize {
        let consumed = amount.min(self.remaining);
        self.remaining -= consumed;
        consumed
    }

    /// Releases previously consumed capacity.
    ///
    /// # Parameters
    ///
    /// - `amount`: Amount to return to this budget.
    ///
    /// # Returns
    ///
    /// `Ok(())` after increasing the remaining capacity.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidRelease`] and leaves the budget unchanged when the
    /// amount exceeds the capacity currently consumed.
    #[inline]
    pub fn release(&mut self, amount: usize) -> Result<(), InvalidRelease> {
        let used = self.used();
        if amount > used {
            Err(InvalidRelease::new(used, amount))
        } else {
            self.remaining += amount;
            Ok(())
        }
    }

    /// Discards all remaining capacity.
    ///
    /// # Returns
    ///
    /// The amount discarded by this operation.
    #[inline]
    pub fn exhaust(&mut self) -> usize {
        let remaining = self.remaining;
        self.remaining = 0;
        remaining
    }
}
