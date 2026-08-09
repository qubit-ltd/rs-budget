// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Structured facts for a failed point limit check.

use core::fmt;
use std::error::Error;

use crate::ResourceLimit;

/// Facts from an observation that exceeded a finite resource limit.
///
/// `R` is the caller-defined resource value retained for diagnostics. The
/// quantity is always an exact `u64`; this immutable error does not mutate or
/// synchronize any budget.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained for diagnostics.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LimitExceeded<R> {
    /// Resource value associated with the rejected observation.
    resource: R,

    /// Finite limit that the observation exceeded.
    limit: ResourceLimit,

    /// Exact observed quantity that exceeded the limit.
    observed: u64,
}

impl<R> LimitExceeded<R> {
    /// Creates structured limit-exceeded facts.
    ///
    /// # Parameters
    ///
    /// * `resource` - Resource value associated with the observation.
    /// * `limit` - Finite limit that was exceeded.
    /// * `observed` - Exact observed quantity.
    ///
    /// # Returns
    ///
    /// An immutable error fact with no state changes.
    #[inline]
    pub const fn new(resource: R, limit: ResourceLimit, observed: u64) -> Self {
        Self {
            resource,
            limit,
            observed,
        }
    }

    /// Returns the resource by reference.
    #[inline(always)]
    pub const fn resource(&self) -> &R {
        &self.resource
    }

    /// Consumes the error and returns its resource.
    #[inline(always)]
    pub fn into_resource(self) -> R {
        self.resource
    }

    /// Returns the finite limit.
    #[inline(always)]
    pub const fn limit(&self) -> ResourceLimit {
        self.limit
    }

    /// Returns the exact rejected observation.
    #[inline(always)]
    pub const fn observed(&self) -> u64 {
        self.observed
    }
}

impl<R: fmt::Debug> fmt::Display for LimitExceeded<R> {
    /// Formats the rejected observation and its finite limit.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "resource {:?} observed {} exceeds limit {}",
            self.resource,
            self.observed,
            self.limit.maximum(),
        )
    }
}

impl<R: fmt::Debug> Error for LimitExceeded<R> {}
