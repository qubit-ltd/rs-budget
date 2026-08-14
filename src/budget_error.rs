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

use crate::Observation;

/// Structured facts describing a resource constraint failure.
///
/// Point checks use [`Self::LimitExceeded`], and cumulative budgets use
/// [`Self::Insufficient`]. Releasable pool release failures use the separate
/// [`crate::ResourceReleaseError`] type because they are not budget failures.
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
        "resource {resource:?} measured {observed}, exceeding the maximum of {maximum:?}"
    )]
    LimitExceeded {
        /// Resource associated with the failed point check.
        resource: R,
        /// Observed point measurement or safe lower bound.
        observed: Observation<Q>,
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
}

impl<R, Q> BudgetError<R, Q>
where
    Q: Copy + Debug,
{
    /// Returns the resource associated with this failure.
    #[must_use]
    #[inline(always)]
    pub const fn resource(&self) -> &R {
        match self {
            Self::LimitExceeded { resource, .. }
            | Self::Insufficient { resource, .. } => resource,
        }
    }

    /// Consumes this error and returns its associated resource.
    #[inline(always)]
    #[must_use]
    pub fn into_resource(self) -> R {
        match self {
            Self::LimitExceeded { resource, .. }
            | Self::Insufficient { resource, .. } => resource,
        }
    }

    /// Returns the cumulative limit for budget and pool failures.
    ///
    /// Returns `Some(limit)` for [`Self::Insufficient`], or `None` for a
    /// point-limit failure.
    #[must_use]
    #[inline(always)]
    pub const fn limit(&self) -> Option<Q> {
        match self {
            Self::LimitExceeded { .. } => None,
            Self::Insufficient { limit, .. } => Some(*limit),
        }
    }

    /// Returns the observation for a point-limit failure.
    ///
    /// Returns `Some(observed)` for [`Self::LimitExceeded`], or `None` for a
    /// cumulative-budget or pool failure.
    #[must_use]
    #[inline(always)]
    pub const fn observation(&self) -> Option<Observation<Q>> {
        match self {
            Self::LimitExceeded { observed, .. } => Some(*observed),
            Self::Insufficient { .. } => None,
        }
    }

    /// Returns the exact point measurement when the observation is exact.
    #[inline(always)]
    #[must_use]
    pub const fn exact_observed(&self) -> Option<Q> {
        match self.observation() {
            Some(Observation::Exact(value)) => Some(value),
            Some(Observation::AtLeast(_)) | None => None,
        }
    }

    /// Returns the safe lower bound of a point measurement.
    #[inline(always)]
    #[must_use]
    pub const fn observed_lower_bound(&self) -> Option<Q> {
        match self.observation() {
            Some(observed) => Some(observed.lower_bound()),
            None => None,
        }
    }

    /// Returns the configured maximum for a point-limit failure.
    ///
    /// Returns `Some(maximum)` for [`Self::LimitExceeded`], or `None` for a
    /// cumulative-budget or pool failure.
    #[inline(always)]
    #[must_use]
    pub const fn maximum(&self) -> Option<Q> {
        match self {
            Self::LimitExceeded { maximum, .. } => Some(*maximum),
            Self::Insufficient { .. } => None,
        }
    }

    /// Returns the remaining capacity for a cumulative-budget failure.
    ///
    /// Returns `Some(remaining)` for [`Self::Insufficient`], including failed
    /// pool acquisitions, or `None` for a point-limit failure.
    #[inline(always)]
    #[must_use]
    pub const fn remaining(&self) -> Option<Q> {
        match self {
            Self::Insufficient { remaining, .. } => Some(*remaining),
            Self::LimitExceeded { .. } => None,
        }
    }

    /// Returns the requested quantity for a cumulative budget failure.
    ///
    /// Returns `Some(requested)` for [`Self::Insufficient`], or `None` for a
    /// point-limit failure.
    #[inline(always)]
    #[must_use]
    pub const fn requested(&self) -> Option<Q> {
        match self {
            Self::LimitExceeded { .. } => None,
            Self::Insufficient { requested, .. } => Some(*requested),
        }
    }
}
