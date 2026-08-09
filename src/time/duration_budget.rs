// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines finite budgets for explicitly measured active durations.

use std::time::Duration;

use super::DurationBudgetError;

/// A finite, monotonic budget for durations explicitly submitted by callers.
///
/// This type never reads a clock. Operation code decides which measured
/// durations count and submits them through [`Self::try_consume`]. Waiting,
/// queueing and backoff do not consume this budget automatically; use
/// [`super::TimeBudget`] for a continuous deadline.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained for diagnostics.
#[must_use]
#[derive(Debug, PartialEq, Eq)]
pub struct DurationBudget<R> {
    /// Resource value retained in consumption errors.
    resource: R,

    /// Finite maximum active duration of the budget.
    limit: Duration,

    /// Duration that has not yet been consumed.
    remaining: Duration,
}

impl<R> DurationBudget<R> {
    /// Creates a zero-used finite duration budget.
    ///
    /// # Parameters
    ///
    /// * `resource` - Domain resource value retained in errors.
    /// * `limit` - Finite maximum active duration.
    ///
    /// # Returns
    ///
    /// A budget whose remaining duration equals `limit`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use qubit_budget::DurationBudget;
    ///
    /// let mut budget = DurationBudget::new("active work", Duration::from_secs(5));
    /// budget.try_consume(Duration::from_secs(2)).expect("two seconds should fit");
    /// assert_eq!(budget.remaining(), Duration::from_secs(3));
    /// ```
    #[inline]
    pub const fn new(resource: R, limit: Duration) -> Self {
        Self {
            resource,
            limit,
            remaining: limit,
        }
    }

    /// Checks whether a complete duration consumption would fit.
    ///
    /// # Parameters
    ///
    /// * `duration` - Explicit duration to check.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the duration fits, otherwise a structured error with no
    /// state mutation.
    ///
    /// # Errors
    ///
    /// Returns [`DurationBudgetError`] when `duration` exceeds the remaining
    /// duration.
    pub fn check_available(
        &self,
        duration: Duration,
    ) -> Result<(), DurationBudgetError<R>>
    where
        R: Clone,
    {
        if duration <= self.remaining {
            Ok(())
        } else {
            Err(DurationBudgetError::new(
                self.resource.clone(),
                self.limit,
                self.remaining,
                duration,
            ))
        }
    }

    /// Consumes a duration atomically when it fits.
    ///
    /// # Parameters
    ///
    /// * `duration` - Explicit active duration to consume.
    ///
    /// # Returns
    ///
    /// `Ok(())` after subtraction, or a structured error while leaving the
    /// remaining duration unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`DurationBudgetError`] when `duration` exceeds the remaining
    /// duration. The budget remains unchanged in that case.
    #[inline]
    pub fn try_consume(
        &mut self,
        duration: Duration,
    ) -> Result<(), DurationBudgetError<R>>
    where
        R: Clone,
    {
        self.check_available(duration)?;
        self.remaining -= duration;
        Ok(())
    }

    /// Consumes as much of the requested duration as remains available.
    ///
    /// # Parameters
    ///
    /// * `requested` - Maximum duration to consume.
    ///
    /// # Returns
    ///
    /// The exact duration consumed, equal to the smaller of `requested` and
    /// the current remaining duration.
    #[inline]
    pub fn consume_available(&mut self, requested: Duration) -> Duration {
        let consumed = requested.min(self.remaining);
        self.remaining -= consumed;
        consumed
    }

    /// Returns the associated resource.
    #[inline(always)]
    pub const fn resource(&self) -> &R {
        &self.resource
    }

    /// Returns the finite duration limit.
    #[inline(always)]
    pub const fn limit(&self) -> Duration {
        self.limit
    }

    /// Returns remaining duration.
    #[inline(always)]
    pub const fn remaining(&self) -> Duration {
        self.remaining
    }

    /// Returns explicitly consumed duration.
    #[inline(always)]
    pub const fn used(&self) -> Duration {
        self.limit.saturating_sub(self.remaining)
    }
}
