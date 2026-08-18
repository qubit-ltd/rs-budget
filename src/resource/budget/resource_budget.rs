// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines monotonic finite resource budgets.

use crate::resource::BudgetGroupError;
use crate::resource::InsufficientBudgetError;
use crate::resource::MeasuredBudgetError;
use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;

/// A finite, non-releasable resource budget.
///
/// The budget stores remaining capacity and only subtracts after a request has
/// been checked. Therefore `used()` is computed as `limit - remaining` and no
/// cumulative addition can overflow.
/// Callers represent an unconfigured dimension with
/// `Option<ResourceBudget<R>> = None` rather than constructing an unlimited
/// object.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained for diagnostics.
/// * `Q` - Exact unsigned quantity used for the limit and accounting.
///
/// A budget is intentionally not cloneable because cloning would create a
/// second independently consumable copy of the same finite allowance.
///
/// # Examples
///
/// ```
/// use qubit_budget::ResourceBudget;
///
/// let mut budget = ResourceBudget::new("response bytes", 8_u64);
/// budget.try_consume(3).expect("three bytes should fit");
/// assert_eq!(budget.used(), 3);
/// assert_eq!(budget.remaining(), 5);
/// ```
///
/// ```compile_fail
/// use qubit_budget::ResourceBudget;
/// let budget = ResourceBudget::new("bytes", 8_u64);
/// let _copy = budget.clone();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct ResourceBudget<R, Q = u64>
where
    Q: ResourceQuantity,
{
    /// Finite maximum capacity of the budget.
    limit: ResourceLimit<R, Q>,

    /// Capacity that has not yet been consumed.
    remaining: Q,
}

impl<R, Q> ResourceBudget<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates a zero-used finite budget.
    ///
    /// # Parameters
    ///
    /// * `resource` - Domain resource value retained in errors.
    /// * `limit` - Finite maximum for this budget.
    ///
    /// # Returns
    ///
    /// A budget whose remaining capacity equals `limit`.
    ///
    /// # Examples
    ///
    /// ```
    /// use qubit_budget::ResourceBudget;
    ///
    /// let mut budget = ResourceBudget::new("bytes", 8_u64);
    /// budget.try_consume(3).expect("three bytes should fit");
    /// assert_eq!(budget.used(), 3);
    /// assert_eq!(budget.remaining(), 5);
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(resource: R, limit: Q) -> Self {
        Self {
            limit: ResourceLimit::new(resource, limit),
            remaining: limit,
        }
    }

    /// Creates a zero-used budget from an immutable resource limit.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource identity and finite maximum for this budget.
    ///
    /// # Returns
    ///
    /// A budget whose remaining capacity equals the limit maximum.
    #[inline]
    #[must_use]
    pub fn from_limit(limit: ResourceLimit<R, Q>) -> Self {
        Self {
            remaining: limit.maximum(),
            limit,
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
    /// [`InsufficientBudgetError`] containing the resource, limit and
    /// pre-failure balance. This method never changes the budget.
    ///
    /// # Errors
    ///
    /// Returns [`InsufficientBudgetError`] when `amount` exceeds the
    /// remaining capacity.
    pub fn check_available(&self, amount: Q) -> Result<(), InsufficientBudgetError<R, Q>>
    where
        R: Clone,
    {
        if amount <= self.remaining {
            Ok(())
        } else {
            Err(InsufficientBudgetError {
                resource: self.limit.resource().clone(),
                limit: self.limit.maximum(),
                remaining: self.remaining,
                requested: amount,
            })
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
    /// Returns [`InsufficientBudgetError`] when `amount` exceeds the
    /// remaining capacity. The budget remains unchanged in that case.
    #[inline]
    pub fn try_consume(&mut self, amount: Q) -> Result<(), InsufficientBudgetError<R, Q>>
    where
        R: Clone,
    {
        self.check_available(amount)?;
        self.remaining = self.remaining - amount;
        Ok(())
    }

    /// Checks a machine-sized consumption request without truncating it.
    ///
    /// # Parameters
    ///
    /// * `amount` - Native quantity to convert and check.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the converted request fits the remaining capacity.
    ///
    /// # Errors
    ///
    /// Returns [`MeasuredBudgetError::Quantity`] when `amount` cannot be
    /// represented by `Q`, or [`MeasuredBudgetError::Budget`] when it exceeds
    /// the remaining capacity. The budget is unchanged on either failure.
    #[inline]
    pub fn check_available_usize(&self, amount: usize) -> Result<(), MeasuredBudgetError<R, Q>>
    where
        R: Clone,
    {
        let amount = Q::try_from_usize(amount)
            .map_err(|source| MeasuredBudgetError::quantity(self.resource().clone(), source))?;
        self.check_available(amount).map_err(MeasuredBudgetError::from)
    }

    /// Checks a 64-bit consumption request without truncating it.
    ///
    /// # Parameters
    ///
    /// * `amount` - 64-bit quantity to convert and check.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the converted request fits the remaining capacity.
    ///
    /// # Errors
    ///
    /// Returns [`MeasuredBudgetError::Quantity`] when `amount` cannot be
    /// represented by `Q`, or [`MeasuredBudgetError::Budget`] when it exceeds
    /// the remaining capacity. The budget is unchanged on either failure.
    #[inline]
    pub fn check_available_u64(&self, amount: u64) -> Result<(), MeasuredBudgetError<R, Q>>
    where
        R: Clone,
    {
        let amount =
            Q::try_from_u64(amount).map_err(|source| MeasuredBudgetError::quantity(self.resource().clone(), source))?;
        self.check_available(amount).map_err(MeasuredBudgetError::from)
    }

    /// Consumes a machine-sized quantity without truncating it.
    ///
    /// # Parameters
    ///
    /// * `amount` - Native quantity to convert and consume.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the converted quantity is consumed.
    ///
    /// # Errors
    ///
    /// Returns [`MeasuredBudgetError::Quantity`] when `amount` cannot be
    /// represented by `Q`, or [`MeasuredBudgetError::Budget`] when it exceeds
    /// the remaining capacity. The budget is unchanged on either failure.
    #[inline]
    pub fn try_consume_usize(&mut self, amount: usize) -> Result<(), MeasuredBudgetError<R, Q>>
    where
        R: Clone,
    {
        let amount = Q::try_from_usize(amount)
            .map_err(|source| MeasuredBudgetError::quantity(self.resource().clone(), source))?;
        self.try_consume(amount).map_err(MeasuredBudgetError::from)
    }

    /// Consumes a 64-bit quantity without truncating it.
    ///
    /// # Parameters
    ///
    /// * `amount` - 64-bit quantity to convert and consume.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the converted quantity is consumed.
    ///
    /// # Errors
    ///
    /// Returns [`MeasuredBudgetError::Quantity`] when `amount` cannot be
    /// represented by `Q`, or [`MeasuredBudgetError::Budget`] when it exceeds
    /// the remaining capacity. The budget is unchanged on either failure.
    #[inline]
    pub fn try_consume_u64(&mut self, amount: u64) -> Result<(), MeasuredBudgetError<R, Q>>
    where
        R: Clone,
    {
        let amount =
            Q::try_from_u64(amount).map_err(|source| MeasuredBudgetError::quantity(self.resource().clone(), source))?;
        self.try_consume(amount).map_err(MeasuredBudgetError::from)
    }

    /// Atomically consumes the same amount from every budget in a group.
    ///
    /// Every member is checked before any member is changed. This is useful
    /// when one operation must count against both a local and a shared budget.
    ///
    /// # Parameters
    ///
    /// * `budgets` - Ordered group of budgets that must all accept the charge.
    /// * `amount` - Quantity to consume from each budget.
    ///
    /// # Returns
    ///
    /// `Ok(())` after every budget is charged, or a [`BudgetGroupError`]
    /// identifying the first rejecting member. Failure leaves every budget
    /// unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetGroupError`] when any member has insufficient remaining
    /// capacity.
    pub fn try_consume_group(budgets: &mut [&mut Self], amount: Q) -> Result<(), BudgetGroupError<R, Q>>
    where
        R: Clone,
    {
        for (index, budget) in budgets.iter().enumerate() {
            budget
                .check_available(amount)
                .map_err(|source| BudgetGroupError::new(index, source))?;
        }
        for budget in budgets {
            let consumed = budget.consume_available(amount);
            debug_assert_eq!(consumed, amount);
        }
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
    #[must_use]
    pub fn consume_available(&mut self, requested: Q) -> Q {
        let consumed = requested.min(self.remaining);
        self.remaining = self.remaining - consumed;
        consumed
    }

    /// Returns the associated resource.
    #[must_use]
    #[inline(always)]
    pub const fn resource(&self) -> &R {
        self.limit.resource()
    }

    /// Returns the immutable resource limit that configures this budget.
    #[must_use]
    #[inline(always)]
    pub const fn resource_limit(&self) -> &ResourceLimit<R, Q> {
        &self.limit
    }

    /// Returns the finite limit.
    #[must_use]
    #[inline(always)]
    pub const fn limit(&self) -> Q {
        self.limit.maximum()
    }

    /// Returns remaining capacity.
    #[must_use]
    #[inline(always)]
    pub const fn remaining(&self) -> Q {
        self.remaining
    }

    /// Returns the quantity consumed so far.
    #[must_use]
    #[inline(always)]
    pub fn used(&self) -> Q {
        self.limit.maximum() - self.remaining
    }
}
