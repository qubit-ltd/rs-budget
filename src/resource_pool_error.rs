// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Unified errors emitted by finite releasable resource pools.

use thiserror::Error;

use crate::ResourceQuantity;

/// Failure facts for either acquisition or release of a resource pool.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained for diagnostics.
/// * `Q` - Exact unsigned quantity used for the capacity and accounting.
#[must_use]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ResourcePoolError<R, Q = u64>
where
    Q: ResourceQuantity,
{
    /// The requested acquisition exceeds current availability.
    #[error(
        "resource {resource:?} has {available} available, but {requested} was requested"
    )]
    Exhausted {
        /// Resource value associated with the pool.
        resource: R,
        /// Finite capacity limit.
        limit: Q,
        /// Capacity available before the failed request.
        available: Q,
        /// Requested acquisition quantity.
        requested: Q,
    },
    /// The requested release exceeds current occupancy.
    #[error(
        "resource {resource:?} has {in_use} in use, but {requested} was released"
    )]
    InvalidRelease {
        /// Resource value associated with the pool.
        resource: R,
        /// Finite capacity limit.
        limit: Q,
        /// Occupancy before the failed request.
        in_use: Q,
        /// Requested release quantity.
        requested: Q,
    },
}

impl<R, Q> ResourcePoolError<R, Q>
where
    Q: ResourceQuantity,
{
    /// Returns the resource by reference.
    #[inline(always)]
    pub const fn resource(&self) -> &R {
        match self {
            Self::Exhausted { resource, .. }
            | Self::InvalidRelease { resource, .. } => resource,
        }
    }

    /// Consumes the error and returns its resource.
    #[inline(always)]
    pub fn into_resource(self) -> R {
        match self {
            Self::Exhausted { resource, .. }
            | Self::InvalidRelease { resource, .. } => resource,
        }
    }

    /// Returns the finite limit.
    #[inline(always)]
    pub const fn limit(&self) -> Q {
        match self {
            Self::Exhausted { limit, .. }
            | Self::InvalidRelease { limit, .. } => *limit,
        }
    }

    /// Returns availability for exhaustion, or `None` for invalid release.
    ///
    /// `Some` contains the availability observed before an exhausted
    /// acquisition; `None` indicates an invalid release error.
    #[inline(always)]
    pub const fn available(&self) -> Option<Q> {
        match self {
            Self::Exhausted { available, .. } => Some(*available),
            Self::InvalidRelease { .. } => None,
        }
    }

    /// Returns occupancy for invalid release, or `None` for exhaustion.
    ///
    /// `Some` contains the occupancy observed before an invalid release;
    /// `None` indicates an exhausted acquisition error.
    #[inline(always)]
    pub const fn in_use(&self) -> Option<Q> {
        match self {
            Self::Exhausted { .. } => None,
            Self::InvalidRelease { in_use, .. } => Some(*in_use),
        }
    }

    /// Returns the requested quantity.
    #[inline(always)]
    pub const fn requested(&self) -> Q {
        match self {
            Self::Exhausted { requested, .. }
            | Self::InvalidRelease { requested, .. } => *requested,
        }
    }
}
