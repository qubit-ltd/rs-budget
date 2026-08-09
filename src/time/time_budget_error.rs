// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
// =============================================================================
//! Errors emitted by continuous monotonic deadline budgets.

use core::fmt;
use std::time::Duration;

use qubit_clock::MonotonicInstant;
use qubit_clock::TimeError;

/// Failure facts for a continuous deadline check.
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
    pub const fn resource(&self) -> &R {
        match self {
            Self::Clock { resource, .. }
            | Self::Expired { resource, .. }
            | Self::WouldExpire { resource, .. } => resource,
        }
    }

    /// Consumes the error and returns its resource.
    pub fn into_resource(self) -> R {
        match self {
            Self::Clock { resource, .. }
            | Self::Expired { resource, .. }
            | Self::WouldExpire { resource, .. } => resource,
        }
    }

    /// Returns the underlying clock error, when present.
    pub const fn clock_error(&self) -> Option<&TimeError> {
        match self {
            Self::Clock { source, .. } => Some(source),
            Self::Expired { .. } | Self::WouldExpire { .. } => None,
        }
    }

    /// Returns the deadline for deadline-related errors.
    pub const fn deadline(&self) -> Option<MonotonicInstant> {
        match self {
            Self::Expired { deadline, .. }
            | Self::WouldExpire { deadline, .. } => Some(*deadline),
            Self::Clock { .. } => None,
        }
    }

    /// Returns the sampled instant for deadline-related errors.
    pub const fn now(&self) -> Option<MonotonicInstant> {
        match self {
            Self::Expired { now, .. } | Self::WouldExpire { now, .. } => {
                Some(*now)
            }
            Self::Clock { .. } => None,
        }
    }

    /// Returns the prospective duration for a would-expire error.
    pub const fn requested(&self) -> Option<Duration> {
        match self {
            Self::WouldExpire { requested, .. } => Some(*requested),
            Self::Clock { .. } | Self::Expired { .. } => None,
        }
    }
}

impl<R: fmt::Debug> fmt::Display for TimeBudgetError<R> {
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

impl<R: fmt::Debug> std::error::Error for TimeBudgetError<R> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Clock { source, .. } => Some(source),
            Self::Expired { .. } | Self::WouldExpire { .. } => None,
        }
    }
}
