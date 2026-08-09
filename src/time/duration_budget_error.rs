// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors emitted by explicit duration budgets.

use std::time::Duration;

use thiserror::Error;

/// Facts from an explicit duration request that did not fit.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained for diagnostics.
#[must_use]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error(
    "resource {resource:?} requested {requested:?}, but only {remaining:?} of {limit:?} remains"
)]
pub struct DurationBudgetError<R> {
    /// Resource value associated with the failed request.
    resource: R,

    /// Finite duration limit that the request exceeded.
    limit: Duration,

    /// Duration remaining before the failed request.
    remaining: Duration,

    /// Duration requested by the failed operation.
    requested: Duration,
}

impl<R> DurationBudgetError<R> {
    /// Creates structured facts for a failed duration request.
    ///
    /// # Parameters
    ///
    /// * `resource` - Resource value associated with the request.
    /// * `limit` - Finite duration limit that the request exceeded.
    /// * `remaining` - Duration remaining before the request.
    /// * `requested` - Duration requested by the operation.
    ///
    /// # Returns
    ///
    /// An immutable error value containing the supplied facts.
    #[inline]
    pub(crate) const fn new(
        resource: R,
        limit: Duration,
        remaining: Duration,
        requested: Duration,
    ) -> Self {
        Self {
            resource,
            limit,
            remaining,
            requested,
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

    /// Returns the finite duration limit.
    #[inline(always)]
    pub const fn limit(&self) -> Duration {
        self.limit
    }

    /// Returns the remaining duration before the failed request.
    #[inline(always)]
    pub const fn remaining(&self) -> Duration {
        self.remaining
    }

    /// Returns the requested duration that did not fit.
    #[inline(always)]
    pub const fn requested(&self) -> Duration {
        self.requested
    }
}
