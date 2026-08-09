// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines monotonic finite resource budgets.

use crate::ResourceBudgetError;
use crate::ResourceLimit;

/// A finite, non-releasable resource budget.
///
/// The budget stores remaining capacity and only subtracts after a request has
/// been checked. Therefore `used()` is computed as
/// `limit.maximum() - remaining` and no cumulative addition can overflow.
/// Callers represent an unconfigured dimension with
/// `Option<ResourceBudget<R>> = None` rather than constructing an unlimited
/// object.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained for diagnostics.
#[must_use]
#[derive(Debug, PartialEq, Eq)]
pub struct ResourceBudget<R> {
    /// Resource value retained in consumption errors.
    resource: R,

    /// Finite maximum capacity of the budget.
    limit: ResourceLimit,

    /// Capacity that has not yet been consumed.
    remaining: u64,
}

impl<R> ResourceBudget<R> {
    /// Creates a zero-used finite budget.
    ///
    /// # Parameters
    ///
    /// * `resource` - Domain resource value retained in errors.
    /// * `limit` - Finite maximum for this budget.
    ///
    /// # Returns
    ///
    /// A budget whose remaining capacity equals `limit.maximum()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use qubit_budget::{ResourceBudget, ResourceLimit};
    ///
    /// let mut budget = ResourceBudget::new("bytes", ResourceLimit::new(8));
    /// budget.try_consume(3).expect("three bytes should fit");
    /// assert_eq!(budget.used(), 3);
    /// assert_eq!(budget.remaining(), 5);
    /// ```
    #[inline]
    pub const fn new(resource: R, limit: ResourceLimit) -> Self {
        Self {
            resource,
            limit,
            remaining: limit.maximum(),
        }
    }

    /// Checks whether a complete consumption would fit.
    ///
    /// # Parameters
    ///
    /// * `amount` - Quantity that would be consumed.
    ///
    /// # Returns
    ///
    /// `Ok(())` when `amount <= remaining`; otherwise returns
    /// [`ResourceBudgetError`] containing the resource, limit and pre-failure
    /// balance. This method never changes the budget.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceBudgetError`] when `amount` exceeds the remaining
    /// capacity.
    pub fn check_available(
        &self,
        amount: u64,
    ) -> Result<(), ResourceBudgetError<R>>
    where
        R: Clone,
    {
        if amount <= self.remaining {
            Ok(())
        } else {
            Err(ResourceBudgetError::new(
                self.resource.clone(),
                self.limit,
                self.remaining,
                amount,
            ))
        }
    }

    /// Consumes an amount atomically when it fits.
    ///
    /// # Parameters
    ///
    /// * `amount` - Quantity to consume.
    ///
    /// # Returns
    ///
    /// `Ok(())` after subtracting the amount, or a structured error while
    /// leaving `remaining` unchanged when the amount does not fit.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceBudgetError`] when `amount` exceeds the remaining
    /// capacity. The budget remains unchanged in that case.
    #[inline]
    pub fn try_consume(
        &mut self,
        amount: u64,
    ) -> Result<(), ResourceBudgetError<R>>
    where
        R: Clone,
    {
        self.check_available(amount)?;
        self.remaining -= amount;
        Ok(())
    }

    /// Consumes as much of a request as remains available.
    ///
    /// # Parameters
    ///
    /// * `requested` - Maximum quantity the caller wants to consume.
    ///
    /// # Returns
    ///
    /// The exact consumed quantity, equal to `min(requested, remaining)`.
    /// This operation always succeeds and never increases the balance.
    #[inline]
    pub fn consume_available(&mut self, requested: u64) -> u64 {
        let consumed = requested.min(self.remaining);
        self.remaining -= consumed;
        consumed
    }

    /// Returns the associated resource.
    #[inline(always)]
    pub const fn resource(&self) -> &R {
        &self.resource
    }

    /// Returns the finite limit.
    #[inline(always)]
    pub const fn limit(&self) -> ResourceLimit {
        self.limit
    }

    /// Returns remaining capacity.
    #[inline(always)]
    pub const fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Returns the quantity consumed so far.
    #[inline(always)]
    pub const fn used(&self) -> u64 {
        self.limit.maximum() - self.remaining
    }
}
