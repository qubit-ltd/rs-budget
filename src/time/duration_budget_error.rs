// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
// =============================================================================
//! Errors emitted by explicit duration budgets.

use core::fmt;
use std::time::Duration;

/// Facts from an explicit duration request that did not fit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurationBudgetError<R> {
    resource: R,
    limit: Duration,
    remaining: Duration,
    requested: Duration,
}

impl<R> DurationBudgetError<R> {
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
    pub const fn resource(&self) -> &R {
        &self.resource
    }

    /// Consumes the error and returns its resource.
    pub fn into_resource(self) -> R {
        self.resource
    }

    /// Returns the finite duration limit.
    pub const fn limit(&self) -> Duration {
        self.limit
    }

    /// Returns the remaining duration before the failed request.
    pub const fn remaining(&self) -> Duration {
        self.remaining
    }

    /// Returns the requested duration that did not fit.
    pub const fn requested(&self) -> Duration {
        self.requested
    }
}

impl<R: fmt::Debug> fmt::Display for DurationBudgetError<R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "resource {:?} requested {:?}, but only {:?} of {:?} remains",
            self.resource, self.requested, self.remaining, self.limit,
        )
    }
}

impl<R: fmt::Debug> std::error::Error for DurationBudgetError<R> {}
