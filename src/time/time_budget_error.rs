// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors emitted by continuous monotonic deadline budgets.

use core::fmt;
use std::error::Error;
use std::time::Duration;

use qubit_clock::MonotonicInstant;
use qubit_clock::TimeError;

/// Failure facts for a continuous deadline check.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained for diagnostics.
#[must_use]
#[derive(Debug)]
pub enum TimeBudgetError<R> {
    /// The clock rejected a domain or instant operation.
    Clock {
        /// Resource value associated with the deadline.
        resource: R,
        /// Underlying clock failure.
        source: TimeError,
    },
    /// The fixed deadline has already been reached.
    Expired {
        /// Resource value associated with the deadline.
        resource: R,
        /// Fixed deadline.
        deadline: MonotonicInstant,
        /// Current sampled instant.
        now: MonotonicInstant,
    },
    /// A prospective operation would reach or pass the deadline.
    WouldExpire {
        /// Resource value associated with the deadline.
        resource: R,
        /// Fixed deadline.
        deadline: MonotonicInstant,
        /// Current sampled instant.
        now: MonotonicInstant,
        /// Prospective operation duration.
        requested: Duration,
    },
}

impl<R> TimeBudgetError<R> {
    /// Returns the resource by reference.
    #[inline(always)]
    pub const fn resource(&self) -> &R {
        match self {
            Self::Clock { resource, .. }
            | Self::Expired { resource, .. }
            | Self::WouldExpire { resource, .. } => resource,
        }
    }

    /// Consumes the error and returns its resource.
    #[inline(always)]
    pub fn into_resource(self) -> R {
        match self {
            Self::Clock { resource, .. }
            | Self::Expired { resource, .. }
            | Self::WouldExpire { resource, .. } => resource,
        }
    }

    /// Returns the underlying clock error, when present.
    ///
    /// # Returns
    ///
    /// `Some` contains the underlying clock error for [`Self::Clock`]; `None`
    /// is returned for [`Self::Expired`] and [`Self::WouldExpire`].
    #[inline(always)]
    pub const fn clock_error(&self) -> Option<&TimeError> {
        match self {
            Self::Clock { source, .. } => Some(source),
            Self::Expired { .. } | Self::WouldExpire { .. } => None,
        }
    }

    /// Returns the deadline for deadline-related errors.
    ///
    /// # Returns
    ///
    /// `Some` contains the fixed deadline for [`Self::Expired`] and
    /// [`Self::WouldExpire`]; `None` is returned for [`Self::Clock`].
    #[inline(always)]
    pub const fn deadline(&self) -> Option<MonotonicInstant> {
        match self {
            Self::Expired { deadline, .. }
            | Self::WouldExpire { deadline, .. } => Some(*deadline),
            Self::Clock { .. } => None,
        }
    }

    /// Returns the sampled instant for deadline-related errors.
    ///
    /// # Returns
    ///
    /// `Some` contains the sampled instant for [`Self::Expired`] and
    /// [`Self::WouldExpire`]; `None` is returned for [`Self::Clock`].
    #[inline(always)]
    pub const fn now(&self) -> Option<MonotonicInstant> {
        match self {
            Self::Expired { now, .. } | Self::WouldExpire { now, .. } => {
                Some(*now)
            }
            Self::Clock { .. } => None,
        }
    }

    /// Returns the prospective duration for a would-expire error.
    ///
    /// # Returns
    ///
    /// `Some` contains the requested duration for [`Self::WouldExpire`];
    /// `None` is returned for [`Self::Clock`] and [`Self::Expired`].
    #[inline(always)]
    pub const fn requested(&self) -> Option<Duration> {
        match self {
            Self::WouldExpire { requested, .. } => Some(*requested),
            Self::Clock { .. } | Self::Expired { .. } => None,
        }
    }
}

impl<R: fmt::Debug> fmt::Display for TimeBudgetError<R> {
    /// Formats the clock or deadline failure facts.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Clock { resource, source } => {
                write!(
                    formatter,
                    "time budget for {resource:?} failed: {source}"
                )
            }
            Self::Expired {
                resource,
                deadline,
                now,
            } => write!(
                formatter,
                "time budget for {resource:?} expired at {deadline:?}; now is {now:?}",
            ),
            Self::WouldExpire {
                resource,
                deadline,
                now,
                requested,
            } => write!(
                formatter,
                "time budget for {resource:?} at {now:?} cannot fit {requested:?} before {deadline:?}",
            ),
        }
    }
}

impl<R: fmt::Debug> Error for TimeBudgetError<R> {
    /// Returns the underlying clock error for a clock failure.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Clock { source, .. } => Some(source),
            Self::Expired { .. } | Self::WouldExpire { .. } => None,
        }
    }
}
