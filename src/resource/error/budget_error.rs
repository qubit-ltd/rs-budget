// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the aggregate error for finite resource constraints.

use std::fmt::Debug;

use thiserror::Error;

use super::InsufficientBudgetError;
use super::LimitExceededError;
use crate::resource::Observation;

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
///
/// # Examples
///
/// ```
/// use qubit_budget::BudgetError;
/// use qubit_budget::LimitExceededError;
///
/// let error = BudgetError::from(LimitExceededError::exact("depth", 3_u64, 2));
/// assert_eq!(error.configured_limit(), 2);
/// assert_eq!(error.exact_observed(), Some(3));
/// ```
#[must_use]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum BudgetError<R, Q = u64>
where
    Q: Copy + Debug,
{
    /// A point measurement exceeded its configured maximum.
    #[error("resource {resource:?} measured {observed}, exceeding the maximum of {maximum:?}")]
    LimitExceeded {
        /// Resource associated with the failed point check.
        resource: R,
        /// Observed point measurement or safe lower bound.
        observed: Observation<Q>,
        /// Configured inclusive point maximum.
        maximum: Q,
    },

    /// A cumulative consumption request exceeded the remaining capacity.
    #[error("resource {resource:?} requested {requested:?}, but only {remaining:?} of {limit:?} remains")]
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
    /// Returns a borrow of the resource associated with this failure.
    ///
    /// # Returns
    ///
    /// Returns a reference to the original resource without cloning it.
    #[must_use = "the associated resource should be inspected"]
    #[inline]
    pub const fn resource(&self) -> &R {
        match self {
            Self::LimitExceeded { resource, .. } | Self::Insufficient { resource, .. } => resource,
        }
    }

    /// Consumes this error and returns its associated resource by value.
    ///
    /// # Returns
    ///
    /// Returns the original resource value without cloning it.
    #[inline]
    #[must_use = "the associated resource should be inspected"]
    pub fn into_resource(self) -> R {
        match self {
            Self::LimitExceeded { resource, .. } | Self::Insufficient { resource, .. } => resource,
        }
    }

    /// Returns the cumulative limit for a budget or pool failure.
    ///
    /// Returns `Some(limit)` for [`Self::Insufficient`], or `None` for a
    /// point-limit failure.
    ///
    /// # Returns
    ///
    /// Returns `Some(limit)` for [`Self::Insufficient`], or `None` for a
    /// point-limit failure.
    #[inline]
    pub const fn limit(&self) -> Option<Q> {
        match self {
            Self::LimitExceeded { .. } => None,
            Self::Insufficient { limit, .. } => Some(*limit),
        }
    }

    /// Returns the point observation when this is a point-limit failure.
    ///
    /// Returns `Some(observed)` for [`Self::LimitExceeded`], or `None` for a
    /// cumulative-budget or pool failure.
    ///
    /// # Returns
    ///
    /// Returns `Some(observation)` for [`Self::LimitExceeded`], or `None` for
    /// a cumulative-budget or pool failure.
    #[inline]
    pub const fn observation(&self) -> Option<Observation<Q>> {
        match self {
            Self::LimitExceeded { observed, .. } => Some(*observed),
            Self::Insufficient { .. } => None,
        }
    }

    /// Returns the exact point measurement when the observation is exact.
    ///
    /// # Returns
    ///
    /// `Some(value)` is returned for a point-limit failure with
    /// [`Observation::Exact`]. `None` is returned for [`Observation::AtLeast`]
    /// or for an [`Self::Insufficient`] failure, which has no point
    /// observation.
    #[inline]
    pub const fn exact_observed(&self) -> Option<Q> {
        match self.observation() {
            Some(Observation::Exact(value)) => Some(value),
            Some(Observation::AtLeast(_)) | None => None,
        }
    }

    /// Returns the safe lower bound of a point observation.
    ///
    /// # Returns
    ///
    /// Returns `Some(lower_bound)` for a point-limit failure, or `None` for a
    /// cumulative-budget failure.
    #[inline]
    pub const fn observed_lower_bound(&self) -> Option<Q> {
        match self.observation() {
            Some(observed) => Some(observed.lower_bound()),
            None => None,
        }
    }

    /// Returns the configured maximum for a point-limit failure, if any.
    ///
    /// Returns `Some(maximum)` for [`Self::LimitExceeded`], or `None` for a
    /// cumulative-budget or pool failure.
    ///
    /// # Returns
    ///
    /// Returns `Some(maximum)` for [`Self::LimitExceeded`], or `None` for a
    /// cumulative-budget or pool failure.
    #[inline]
    pub const fn maximum(&self) -> Option<Q> {
        match self {
            Self::LimitExceeded { maximum, .. } => Some(*maximum),
            Self::Insufficient { .. } => None,
        }
    }

    /// Returns the remaining capacity for a cumulative-budget failure, if any.
    ///
    /// Returns `Some(remaining)` for [`Self::Insufficient`], including failed
    /// pool acquisitions, or `None` for a point-limit failure.
    ///
    /// # Returns
    ///
    /// Returns `Some(remaining)` for [`Self::Insufficient`], or `None` for a
    /// point-limit failure.
    #[inline]
    pub const fn remaining(&self) -> Option<Q> {
        match self {
            Self::Insufficient { remaining, .. } => Some(*remaining),
            Self::LimitExceeded { .. } => None,
        }
    }

    /// Returns the requested quantity for a cumulative-budget failure, if any.
    ///
    /// Returns `Some(requested)` for [`Self::Insufficient`], or `None` for a
    /// point-limit failure.
    ///
    /// # Returns
    ///
    /// Returns `Some(requested)` for [`Self::Insufficient`], or `None` for a
    /// point-limit failure.
    #[inline]
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
    /// Returns the configured limit for either point or cumulative failure.
    ///
    /// # Returns
    ///
    /// Returns the point maximum for [`Self::LimitExceeded`] or cumulative
    /// limit for [`Self::Insufficient`].
    #[must_use]
    #[inline]
    pub const fn configured_limit(&self) -> Q {
        match self {
            Self::LimitExceeded { maximum, .. } => *maximum,
            Self::Insufficient { limit, .. } => *limit,
        }
    }

    /// Returns cumulative usage before a failed request, when available.
    ///
    /// # Returns
    ///
    /// Returns `Some(used)` for [`Self::Insufficient`], or `None` for a
    /// point-limit failure.
    ///
    /// The public fields must satisfy `remaining <= limit`.
    ///
    /// # Panics
    ///
    /// May panic when overflow checks are enabled if manually constructed or
    /// modified fields violate `remaining <= limit`. With overflow checks
    /// disabled, the returned diagnostic value is not meaningful for such
    /// invalid fields.
    #[inline]
    pub fn used(&self) -> Option<Q> {
        match self {
            Self::LimitExceeded { .. } => None,
            Self::Insufficient { limit, remaining, .. } => Some(*limit - *remaining),
        }
    }
}

impl<R, Q> From<LimitExceededError<R, Q>> for BudgetError<R, Q>
where
    Q: Copy + Debug,
{
    /// Converts a precise point-limit failure into an aggregate error.
    ///
    /// # Parameters
    ///
    /// * `error` - Precise point-limit failure to convert.
    ///
    /// # Returns
    ///
    /// Converts a precise point-limit failure into an aggregate error.
    #[inline]
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
    ///
    /// # Parameters
    ///
    /// * `error` - Precise cumulative-budget failure to convert.
    ///
    /// # Returns
    ///
    /// Converts a precise cumulative-budget failure into an aggregate error.
    #[inline]
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
