// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines finite budgets for explicitly measured active durations.

use std::time::Duration;

use crate::resource::InsufficientBudgetError;
use crate::resource::ResourceLimit;

/// A finite, monotonic budget for durations explicitly submitted by callers.
///
/// This type never reads a clock. Operation code decides which measured
/// durations count and submits them through [`Self::try_consume`]. Waiting,
/// queueing and backoff do not consume this budget automatically; use
/// the clock-backed `TimeBudget` for a continuous deadline.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained for diagnostics.
#[derive(Debug, PartialEq, Eq)]
pub struct DurationBudget<R> {
    /// Finite maximum active duration of the budget.
    limit: ResourceLimit<R, Duration>,

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
    #[must_use]
    pub const fn new(resource: R, limit: Duration) -> Self {
        Self {
            limit: ResourceLimit::new(resource, limit),
            remaining: limit,
        }
    }

    /// Creates a zero-used duration budget from an immutable resource limit.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource identity and finite duration for this budget.
    ///
    /// # Returns
    ///
    /// A budget whose remaining duration equals the limit maximum.
    #[inline]
    #[must_use]
    pub fn from_limit(limit: ResourceLimit<R, Duration>) -> Self {
        let remaining = limit.maximum();
        Self { limit, remaining }
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
    /// Returns [`InsufficientBudgetError`] when `duration` exceeds the
    /// remaining duration.
    pub fn check_available(&self, duration: Duration) -> Result<(), InsufficientBudgetError<R, Duration>>
    where
        R: Clone,
    {
        if duration <= self.remaining {
            Ok(())
        } else {
            Err(InsufficientBudgetError {
                resource: self.limit.resource().clone(),
                limit: self.limit.maximum(),
                remaining: self.remaining,
                requested: duration,
            })
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
    /// Returns [`InsufficientBudgetError`] when `duration` exceeds the
    /// remaining duration. The budget remains unchanged in that case.
    #[inline]
    pub fn try_consume(&mut self, duration: Duration) -> Result<(), InsufficientBudgetError<R, Duration>>
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
    #[must_use]
    pub fn consume_available(&mut self, requested: Duration) -> Duration {
        let consumed = requested.min(self.remaining);
        self.remaining -= consumed;
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
    pub const fn resource_limit(&self) -> &ResourceLimit<R, Duration> {
        &self.limit
    }

    /// Returns the finite duration limit.
    #[must_use]
    #[inline(always)]
    pub const fn limit(&self) -> Duration {
        self.limit.maximum()
    }

    /// Returns remaining duration.
    #[must_use]
    #[inline(always)]
    pub const fn remaining(&self) -> Duration {
        self.remaining
    }

    /// Returns explicitly consumed duration.
    #[must_use]
    #[inline(always)]
    pub const fn used(&self) -> Duration {
        self.limit.maximum().saturating_sub(self.remaining)
    }
}
