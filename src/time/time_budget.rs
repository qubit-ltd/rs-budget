// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
// =============================================================================
//! Defines continuous finite monotonic deadline budgets.

use std::time::Duration;

use qubit_clock::MonotonicClock;
use qubit_clock::MonotonicInstant;

use super::TimeBudgetError;

/// A continuous finite deadline budget in an injected monotonic clock domain.
///
/// The deadline advances naturally as the clock advances, so operation time,
/// waiting, queueing and backoff all consume the same end-to-end budget. This
/// type has no mutable state or explicit charge counter. An unconfigured
/// deadline is represented by `Option<TimeBudget<R, C>> = None`.
#[must_use]
#[derive(Debug)]
pub struct TimeBudget<R, C> {
    resource: R,
    clock: C,
    started_at: MonotonicInstant,
    deadline: MonotonicInstant,
}

impl<R: Clone, C: MonotonicClock> TimeBudget<R, C> {
    /// Creates a deadline after a relative duration.
    ///
    /// # Parameters
    ///
    /// * `resource` - Domain resource value retained in errors.
    /// * `clock` - Monotonic clock used for every later sample.
    /// * `duration` - Finite duration from the construction sample.
    ///
    /// # Returns
    ///
    /// A deadline budget, or a clock error when the deadline instant cannot be
    /// represented. No budget is returned on error.
    pub fn for_duration(
        resource: R,
        clock: C,
        duration: Duration,
    ) -> Result<Self, TimeBudgetError<R>> {
        let started_at = clock.now();
        let deadline = started_at.checked_add(duration).map_err(|source| {
            TimeBudgetError::Clock {
                resource: resource.clone(),
                source,
            }
        })?;
        Ok(Self {
            resource,
            clock,
            started_at,
            deadline,
        })
    }

    /// Creates a deadline from an absolute same-domain instant.
    ///
    /// # Parameters
    ///
    /// * `resource` - Domain resource value retained in errors.
    /// * `clock` - Monotonic clock whose domain must match `deadline`.
    /// * `deadline` - Fixed absolute deadline; it may already be expired.
    ///
    /// # Returns
    ///
    /// A deadline budget, or a clock-domain error when the instant belongs to
    /// another clock domain.
    pub fn until(
        resource: R,
        clock: C,
        deadline: MonotonicInstant,
    ) -> Result<Self, TimeBudgetError<R>> {
        deadline.validate_domain(clock.domain()).map_err(|source| {
            TimeBudgetError::Clock {
                resource: resource.clone(),
                source,
            }
        })?;
        let started_at = clock.now();
        Ok(Self {
            resource,
            clock,
            started_at,
            deadline,
        })
    }

    /// Returns the associated resource.
    pub const fn resource(&self) -> &R {
        &self.resource
    }

    /// Returns the instant sampled at construction.
    pub const fn started_at(&self) -> MonotonicInstant {
        self.started_at
    }

    /// Returns the fixed deadline.
    pub const fn deadline(&self) -> MonotonicInstant {
        self.deadline
    }

    /// Returns elapsed time since construction.
    ///
    /// # Returns
    ///
    /// The current clock duration since `started_at`, or a structured clock
    /// error if same-domain arithmetic fails.
    pub fn elapsed(&self) -> Result<Duration, TimeBudgetError<R>> {
        self.clock
            .now()
            .duration_since(self.started_at)
            .map_err(|source| TimeBudgetError::Clock {
                resource: self.resource.clone(),
                source,
            })
    }

    /// Returns the non-negative time remaining before the deadline.
    ///
    /// # Returns
    ///
    /// `Duration::ZERO` once the deadline is reached; otherwise the exact
    /// duration until it. Same-domain arithmetic errors are returned with the
    /// resource value.
    pub fn remaining(&self) -> Result<Duration, TimeBudgetError<R>> {
        let now = self.clock.now();
        if now >= self.deadline {
            Ok(Duration::ZERO)
        } else {
            self.deadline.duration_since(now).map_err(|source| {
                TimeBudgetError::Clock {
                    resource: self.resource.clone(),
                    source,
                }
            })
        }
    }

    /// Reports whether the current instant has reached the deadline.
    pub fn is_expired(&self) -> Result<bool, TimeBudgetError<R>> {
        Ok(self.clock.now() >= self.deadline)
    }

    /// Checks that the deadline has not already been reached.
    ///
    /// # Returns
    ///
    /// `Ok(())` before the deadline, or [`TimeBudgetError::Expired`] with the
    /// current sample and fixed deadline after it.
    pub fn check(&self) -> Result<(), TimeBudgetError<R>> {
        let now = self.clock.now();
        if now >= self.deadline {
            Err(TimeBudgetError::Expired {
                resource: self.resource.clone(),
                deadline: self.deadline,
                now,
            })
        } else {
            Ok(())
        }
    }

    /// Checks that an operation would finish strictly before the deadline.
    ///
    /// # Parameters
    ///
    /// * `duration` - Prospective operation duration.
    ///
    /// # Returns
    ///
    /// `Ok(())` when `now + duration < deadline`; an expired error when the
    /// budget is already expired; a would-expire error when the prospective
    /// end reaches the deadline; or a clock error on instant overflow.
    pub fn check_after(
        &self,
        duration: Duration,
    ) -> Result<(), TimeBudgetError<R>> {
        let now = self.clock.now();
        if now >= self.deadline {
            return Err(TimeBudgetError::Expired {
                resource: self.resource.clone(),
                deadline: self.deadline,
                now,
            });
        }
        let end = now.checked_add(duration).map_err(|source| {
            TimeBudgetError::Clock {
                resource: self.resource.clone(),
                source,
            }
        })?;
        if end >= self.deadline {
            Err(TimeBudgetError::WouldExpire {
                resource: self.resource.clone(),
                deadline: self.deadline,
                now,
                requested: duration,
            })
        } else {
            Ok(())
        }
    }
}
