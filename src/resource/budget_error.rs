// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines precise and aggregate errors for finite resource constraints.
// qubit-style: allow multiple-public-types

use std::fmt::Debug;

use thiserror::Error;

use crate::Observation;

/// Structured facts for a point measurement that exceeded its maximum.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained for diagnostics.
/// * `Q` - Copyable measurement value used by the failed constraint.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error(
    "resource {resource:?} measured {observed}, exceeding the maximum of {maximum:?}"
)]
pub struct LimitExceededError<R, Q = u64>
where
    Q: Copy + Debug,
{
    /// Resource associated with the failed point check.
    pub resource: R,
    /// Observed point measurement or safe lower bound.
    pub observed: Observation<Q>,
    /// Configured inclusive point maximum.
    pub maximum: Q,
}

impl<R, Q> LimitExceededError<R, Q>
where
    Q: Copy + Debug,
{
    /// Returns the resource associated with this failure.
    #[must_use]
    #[inline(always)]
    pub const fn resource(&self) -> &R {
        &self.resource
    }

    /// Consumes this error and returns its associated resource.
    #[must_use]
    #[inline(always)]
    pub fn into_resource(self) -> R {
        self.resource
    }

    /// Returns the observed point measurement or safe lower bound.
    #[must_use]
    #[inline(always)]
    pub const fn observation(&self) -> Observation<Q> {
        self.observed
    }

    /// Returns the exact point measurement when the observation is exact.
    #[must_use]
    #[inline(always)]
    pub const fn exact_observed(&self) -> Option<Q> {
        match self.observed {
            Observation::Exact(value) => Some(value),
            Observation::AtLeast(_) => None,
        }
    }

    /// Returns the safe lower bound of the observed point measurement.
    #[must_use]
    #[inline(always)]
    pub const fn observed_lower_bound(&self) -> Q {
        self.observed.lower_bound()
    }

    /// Returns the configured inclusive point maximum.
    #[must_use]
    #[inline(always)]
    pub const fn maximum(&self) -> Q {
        self.maximum
    }
}

/// Structured facts for a cumulative request that exceeded remaining capacity.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained for diagnostics.
/// * `Q` - Copyable measurement value used by the failed constraint.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error(
    "resource {resource:?} requested {requested:?}, but only {remaining:?} of {limit:?} remains"
)]
pub struct InsufficientBudgetError<R, Q = u64>
where
    Q: Copy + Debug,
{
    /// Resource associated with the failed consumption request.
    pub resource: R,
    /// Configured finite limit.
    pub limit: Q,
    /// Capacity remaining before the failed request.
    pub remaining: Q,
    /// Quantity requested by the failed operation.
    pub requested: Q,
}

impl<R, Q> InsufficientBudgetError<R, Q>
where
    Q: Copy + Debug,
{
    /// Returns the resource associated with this failure.
    #[must_use]
    #[inline(always)]
    pub const fn resource(&self) -> &R {
        &self.resource
    }

    /// Consumes this error and returns its associated resource.
    #[must_use]
    #[inline(always)]
    pub fn into_resource(self) -> R {
        self.resource
    }

    /// Returns the configured finite limit.
    #[must_use]
    #[inline(always)]
    pub const fn limit(&self) -> Q {
        self.limit
    }

    /// Returns the capacity remaining before the failed request.
    #[must_use]
    #[inline(always)]
    pub const fn remaining(&self) -> Q {
        self.remaining
    }

    /// Returns the quantity requested by the failed operation.
    #[must_use]
    #[inline(always)]
    pub const fn requested(&self) -> Q {
        self.requested
    }
}

impl<R, Q> InsufficientBudgetError<R, Q>
where
    Q: crate::ResourceQuantity,
{
    /// Returns the amount already consumed before the failed request.
    #[must_use]
    #[inline]
    pub fn used(&self) -> Q {
        self.limit - self.remaining
    }
}

/// Aggregate error for APIs that can perform both point and cumulative checks.
///
/// APIs with a single failure mode return [`LimitExceededError`] or
/// [`InsufficientBudgetError`] directly. This type remains the common carrier
/// for composite operations and measured-value errors. Releasable pool release
/// failures use the separate [`crate::ResourceReleaseError`] type.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained for diagnostics.
/// * `Q` - Copyable measurement value used by the failed constraint.
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

impl<R, Q> BudgetError<R, Q>
where
    Q: crate::ResourceQuantity,
{
    /// Returns the configured limit for either point or cumulative failures.
    #[must_use]
    #[inline]
    pub const fn configured_limit(&self) -> Q {
        match self {
            Self::LimitExceeded { maximum, .. } => *maximum,
            Self::Insufficient { limit, .. } => *limit,
        }
    }

    /// Returns cumulative usage before a failed request, when applicable.
    #[must_use]
    #[inline]
    pub fn used(&self) -> Option<Q> {
        match self {
            Self::LimitExceeded { .. } => None,
            Self::Insufficient {
                limit, remaining, ..
            } => Some(*limit - *remaining),
        }
    }
}

impl<R, Q> From<LimitExceededError<R, Q>> for BudgetError<R, Q>
where
    Q: Copy + Debug,
{
    /// Converts a precise point-limit failure into an aggregate error.
    #[inline(always)]
    fn from(error: LimitExceededError<R, Q>) -> Self {
        let LimitExceededError {
            resource,
            observed,
            maximum,
        } = error;
        Self::LimitExceeded {
            resource,
            observed,
            maximum,
        }
    }
}

impl<R, Q> From<InsufficientBudgetError<R, Q>> for BudgetError<R, Q>
where
    Q: Copy + Debug,
{
    /// Converts a precise cumulative-budget failure into an aggregate error.
    #[inline(always)]
    fn from(error: InsufficientBudgetError<R, Q>) -> Self {
        let InsufficientBudgetError {
            resource,
            limit,
            remaining,
            requested,
        } = error;
        Self::Insufficient {
            resource,
            limit,
            remaining,
            requested,
        }
    }
}
