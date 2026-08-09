// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors emitted by finite cumulative resource budgets.

use core::fmt;
use std::error::Error;

use crate::ResourceLimit;

/// Facts from a resource consumption request that did not fit.
///
/// The stored `remaining` value is the balance before the failed request.
/// Errors are constructed before any budget mutation, so a failed operation is
/// failure-atomic.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained for diagnostics.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceBudgetError<R> {
    /// Resource value associated with the failed request.
    resource: R,

    /// Finite limit that the request exceeded.
    limit: ResourceLimit,

    /// Capacity remaining before the failed request.
    remaining: u64,

    /// Quantity requested by the failed operation.
    requested: u64,
}

impl<R> ResourceBudgetError<R> {
    /// Creates structured facts for a failed resource request.
    ///
    /// # Parameters
    ///
    /// * `resource` - Resource value associated with the request.
    /// * `limit` - Finite limit that the request exceeded.
    /// * `remaining` - Capacity remaining before the request.
    /// * `requested` - Quantity requested by the operation.
    ///
    /// # Returns
    ///
    /// An immutable error value containing the supplied facts.
    #[inline]
    pub(crate) const fn new(
        resource: R,
        limit: ResourceLimit,
        remaining: u64,
        requested: u64,
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

    /// Returns the finite limit.
    #[inline(always)]
    pub const fn limit(&self) -> ResourceLimit {
        self.limit
    }

    /// Returns the balance before the failed request.
    #[inline(always)]
    pub const fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Returns the requested quantity that did not fit.
    #[inline(always)]
    pub const fn requested(&self) -> u64 {
        self.requested
    }
}

impl<R: fmt::Debug> fmt::Display for ResourceBudgetError<R> {
    /// Formats the failed request and the capacity that remained.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "resource {:?} requested {}, but only {} of {} remains",
            self.resource,
            self.requested,
            self.remaining,
            self.limit.maximum(),
        )
    }
}

impl<R: fmt::Debug> Error for ResourceBudgetError<R> {}
