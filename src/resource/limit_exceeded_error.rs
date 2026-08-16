// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines point-limit failures for finite resource constraints.

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
