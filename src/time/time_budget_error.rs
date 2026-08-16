// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors emitted by continuous monotonic deadline budgets.

use std::time::Duration;

use qubit_clock::MonotonicInstant;
use qubit_clock::TimeError;
use thiserror::Error;

/// Failure facts for a continuous deadline check.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained for diagnostics.
#[derive(Debug, Error)]
pub enum TimeBudgetError<R> {
    /// The clock rejected a domain or instant operation.
    #[error("time budget for {resource:?} failed: {source}")]
    Clock {
        /// Resource value associated with the deadline.
        resource: R,
        /// Underlying clock failure.
        #[source]
        source: TimeError,
    },
    /// The fixed deadline has already been reached.
    #[error(
        "time budget for {resource:?} expired at {deadline:?}; now is {now:?}"
    )]
    Expired {
        /// Resource value associated with the deadline.
        resource: R,
        /// Fixed deadline.
        deadline: MonotonicInstant,
        /// Current sampled instant.
        now: MonotonicInstant,
    },
    /// A prospective operation would reach or pass the deadline.
    #[error(
        "time budget for {resource:?} at {now:?} cannot fit {requested:?} before {deadline:?}"
    )]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    #[inline(always)]
    pub const fn requested(&self) -> Option<Duration> {
        match self {
            Self::WouldExpire { requested, .. } => Some(*requested),
            Self::Clock { .. } | Self::Expired { .. } => None,
        }
    }
}
