// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
// =============================================================================
//! Unified errors emitted by finite releasable resource pools.

use core::fmt;

use crate::ResourceLimit;

/// Failure facts for either acquisition or release of a resource pool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourcePoolError<R> {
    /// The requested acquisition exceeds current availability.
    Exhausted {
        /// Resource value associated with the pool.
        resource: R,
        /// Finite capacity limit.
        limit: ResourceLimit,
        /// Capacity available before the failed request.
        available: u64,
        /// Requested acquisition quantity.
        requested: u64,
    },
    /// The requested release exceeds current occupancy.
    InvalidRelease {
        /// Resource value associated with the pool.
        resource: R,
        /// Finite capacity limit.
        limit: ResourceLimit,
        /// Occupancy before the failed request.
        in_use: u64,
        /// Requested release quantity.
        requested: u64,
    },
}

impl<R> ResourcePoolError<R> {
    /// Returns the resource by reference.
    pub const fn resource(&self) -> &R {
        match self {
            Self::Exhausted { resource, .. }
            | Self::InvalidRelease { resource, .. } => resource,
        }
    }

    /// Consumes the error and returns its resource.
    pub fn into_resource(self) -> R {
        match self {
            Self::Exhausted { resource, .. }
            | Self::InvalidRelease { resource, .. } => resource,
        }
    }

    /// Returns the finite limit.
    pub const fn limit(&self) -> ResourceLimit {
        match self {
            Self::Exhausted { limit, .. }
            | Self::InvalidRelease { limit, .. } => *limit,
        }
    }

    /// Returns availability for exhaustion, or `None` for invalid release.
    pub const fn available(&self) -> Option<u64> {
        match self {
            Self::Exhausted { available, .. } => Some(*available),
            Self::InvalidRelease { .. } => None,
        }
    }

    /// Returns occupancy for invalid release, or `None` for exhaustion.
    pub const fn in_use(&self) -> Option<u64> {
        match self {
            Self::Exhausted { .. } => None,
            Self::InvalidRelease { in_use, .. } => Some(*in_use),
        }
    }

    /// Returns the requested quantity.
    pub const fn requested(&self) -> u64 {
        match self {
            Self::Exhausted { requested, .. }
            | Self::InvalidRelease { requested, .. } => *requested,
        }
    }
}

impl<R: fmt::Debug> fmt::Display for ResourcePoolError<R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exhausted {
                resource,
                available,
                requested,
                ..
            } => write!(
                formatter,
                "resource {:?} has {} available, but {} was requested",
                resource, available, requested
            ),
            Self::InvalidRelease {
                resource,
                in_use,
                requested,
                ..
            } => write!(
                formatter,
                "resource {:?} has {} in use, but {} was released",
                resource, in_use, requested
            ),
        }
    }
}

impl<R: fmt::Debug> std::error::Error for ResourcePoolError<R> {}
