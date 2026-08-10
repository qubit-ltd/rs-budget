// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines unified errors for finite resource constraints.

use std::fmt::Debug;

use thiserror::Error;

/// Structured facts describing a resource constraint failure.
///
/// Point checks use [`Self::LimitExceeded`], cumulative budgets use
/// [`Self::Insufficient`], and releasable pools use [`Self::InvalidRelease`].
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained for diagnostics.
/// * `Q` - Copyable measurement value used by the failed constraint.
#[must_use]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum BudgetError<R, Q = u64>
where
    Q: Copy + Debug,
{
    /// A point measurement exceeded its configured maximum.
    #[error(
        "resource {resource:?} measured {actual:?}, exceeding the maximum of {maximum:?}"
    )]
    LimitExceeded {
        /// Resource associated with the failed point check.
        resource: R,
        /// Observed point measurement.
        actual: Q,
        /// Configured inclusive point maximum.
        maximum: Q,
    },

    /// A cumulative consumption request exceeded the remaining capacity.
    #[error(
        "resource {resource:?} requested {requested:?}, but only {remaining:?} of {limit:?} remains"
    )]
    Insufficient {
        /// Resource associated with the failed consumption request.
        resource: R,
        /// Configured finite limit.
        limit: Q,
        /// Capacity remaining before the failed request.
        remaining: Q,
        /// Quantity requested by the failed operation.
        requested: Q,
    },

    /// A release request exceeded the amount currently in use.
    #[error(
        "resource {resource:?} has {in_use:?} in use, but {requested:?} was released"
    )]
    InvalidRelease {
        /// Resource associated with the failed release request.
        resource: R,
        /// Configured finite limit.
        limit: Q,
        /// Quantity in use before the failed release.
        in_use: Q,
        /// Quantity requested by the failed operation.
        requested: Q,
    },
}

impl<R, Q> BudgetError<R, Q>
where
    Q: Copy + Debug,
{
    /// Returns the resource associated with this failure.
    #[inline(always)]
    pub const fn resource(&self) -> &R {
        match self {
            Self::LimitExceeded { resource, .. }
            | Self::Insufficient { resource, .. }
            | Self::InvalidRelease { resource, .. } => resource,
        }
    }

    /// Consumes this error and returns its associated resource.
    #[inline(always)]
    pub fn into_resource(self) -> R {
        match self {
            Self::LimitExceeded { resource, .. }
            | Self::Insufficient { resource, .. }
            | Self::InvalidRelease { resource, .. } => resource,
        }
    }

    /// Returns the cumulative limit for budget and pool failures.
    ///
    /// Returns `Some(limit)` for [`Self::Insufficient`] and
    /// [`Self::InvalidRelease`], or `None` for a point-limit failure.
    #[inline(always)]
    pub const fn limit(&self) -> Option<Q> {
        match self {
            Self::LimitExceeded { .. } => None,
            Self::Insufficient { limit, .. }
            | Self::InvalidRelease { limit, .. } => Some(*limit),
        }
    }

    /// Returns the observed measurement for a point-limit failure.
    ///
    /// Returns `Some(actual)` for [`Self::LimitExceeded`], or `None` for a
    /// cumulative-budget or pool failure.
    #[inline(always)]
    pub const fn actual(&self) -> Option<Q> {
        match self {
            Self::LimitExceeded { actual, .. } => Some(*actual),
            Self::Insufficient { .. } | Self::InvalidRelease { .. } => None,
        }
    }

    /// Returns the configured maximum for a point-limit failure.
    ///
    /// Returns `Some(maximum)` for [`Self::LimitExceeded`], or `None` for a
    /// cumulative-budget or pool failure.
    #[inline(always)]
    pub const fn maximum(&self) -> Option<Q> {
        match self {
            Self::LimitExceeded { maximum, .. } => Some(*maximum),
            Self::Insufficient { .. } | Self::InvalidRelease { .. } => None,
        }
    }

    /// Returns the remaining capacity for a cumulative-budget failure.
    ///
    /// Returns `Some(remaining)` for [`Self::Insufficient`], or `None` for a
    /// point-limit or pool failure.
    #[inline(always)]
    pub const fn remaining(&self) -> Option<Q> {
        match self {
            Self::Insufficient { remaining, .. } => Some(*remaining),
            Self::LimitExceeded { .. } | Self::InvalidRelease { .. } => None,
        }
    }

    /// Returns the amount in use for an invalid pool release.
    ///
    /// Returns `Some(in_use)` for [`Self::InvalidRelease`], or `None` for a
    /// point-limit or cumulative-budget failure.
    #[inline(always)]
    pub const fn in_use(&self) -> Option<Q> {
        match self {
            Self::InvalidRelease { in_use, .. } => Some(*in_use),
            Self::LimitExceeded { .. } | Self::Insufficient { .. } => None,
        }
    }

    /// Returns the requested quantity for budget and pool failures.
    ///
    /// Returns `Some(requested)` for [`Self::Insufficient`] and
    /// [`Self::InvalidRelease`], or `None` for a point-limit failure.
    #[inline(always)]
    pub const fn requested(&self) -> Option<Q> {
        match self {
            Self::LimitExceeded { .. } => None,
            Self::Insufficient { requested, .. }
            | Self::InvalidRelease { requested, .. } => Some(*requested),
        }
    }
}
