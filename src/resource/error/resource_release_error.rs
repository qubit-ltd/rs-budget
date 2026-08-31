// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines errors for invalid releasable-pool operations.

use std::fmt::Debug;

use thiserror::Error;

/// Structured facts describing a pool release that exceeds current usage.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Examples
///
/// ```
/// use qubit_budget::ResourcePool;
///
/// let mut pool = ResourcePool::new("workers", 2_u64);
/// pool.try_acquire(1).expect("one worker should fit");
/// let error = pool.release(2).expect_err("only one worker is in use");
/// assert_eq!(error.in_use(), 1);
/// assert_eq!(error.requested(), 2);
/// ```
#[non_exhaustive]
#[must_use]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ResourceReleaseError<R, Q = u64>
where
    Q: Copy + Debug,
{
    /// A release request exceeded the amount currently in use.
    #[error("resource {resource:?} has {in_use:?} in use, but {requested:?} was released")]
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

impl<R, Q> ResourceReleaseError<R, Q>
where
    Q: Copy + Debug,
{
    /// Returns the resource associated with this failure.
    ///
    /// # Returns
    ///
    /// Returns the resource associated with this failure.
    #[must_use]
    #[inline(always)]
    pub const fn resource(&self) -> &R {
        match self {
            Self::InvalidRelease { resource, .. } => resource,
        }
    }

    /// Consumes this error and returns its associated resource.
    ///
    /// # Returns
    ///
    /// Consumes this error and returns its associated resource.
    #[inline(always)]
    #[must_use]
    pub fn into_resource(self) -> R {
        match self {
            Self::InvalidRelease { resource, .. } => resource,
        }
    }

    /// Returns the finite pool limit.
    ///
    /// # Returns
    ///
    /// Returns the finite pool limit.
    #[must_use]
    #[inline(always)]
    pub const fn limit(&self) -> Q {
        match self {
            Self::InvalidRelease { limit, .. } => *limit,
        }
    }

    /// Returns the amount in use before the invalid release.
    ///
    /// # Returns
    ///
    /// Returns the amount in use before the invalid release.
    #[must_use]
    #[inline(always)]
    pub const fn in_use(&self) -> Q {
        match self {
            Self::InvalidRelease { in_use, .. } => *in_use,
        }
    }

    /// Returns the requested release amount.
    ///
    /// # Returns
    ///
    /// Returns the requested release amount.
    #[must_use]
    #[inline(always)]
    pub const fn requested(&self) -> Q {
        match self {
            Self::InvalidRelease { requested, .. } => *requested,
        }
    }
}
