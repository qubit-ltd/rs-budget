// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
// =============================================================================
//! Errors emitted by finite cumulative resource budgets.

use core::fmt;

use crate::ResourceLimit;

/// Facts from a resource consumption request that did not fit.
///
/// The stored `remaining` value is the balance before the failed request.
/// Errors are constructed before any budget mutation, so a failed operation is
/// failure-atomic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceBudgetError<R> {
    resource: R,
    limit: ResourceLimit,
    remaining: u64,
    requested: u64,
}

impl<R> ResourceBudgetError<R> {
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
    pub const fn resource(&self) -> &R {
        &self.resource
    }

    /// Consumes the error and returns its resource.
    pub fn into_resource(self) -> R {
        self.resource
    }

    /// Returns the finite limit.
    pub const fn limit(&self) -> ResourceLimit {
        self.limit
    }

    /// Returns the balance before the failed request.
    pub const fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Returns the requested quantity that did not fit.
    pub const fn requested(&self) -> u64 {
        self.requested
    }
}

impl<R: fmt::Debug> fmt::Display for ResourceBudgetError<R> {
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

impl<R: fmt::Debug> std::error::Error for ResourceBudgetError<R> {}
