// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines cumulative-budget failures for finite resource constraints.

use std::fmt::Debug;

use thiserror::Error;

/// Structured facts for a cumulative request that exceeded remaining capacity.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained for diagnostics.
/// * `Q` - Copyable measurement value used by the failed constraint.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error(
    "resource {resource:?} requested {requested:?}, but only {remaining:?} of {limit:?} remains"
)]
pub struct InsufficientBudgetError<R, Q = u64>
where
    Q: Copy + Debug,
{
    /// Resource associated with the failed consumption request.
    pub resource: R,
    /// Configured finite limit.
    pub limit: Q,
    /// Capacity remaining before the failed request.
    pub remaining: Q,
    /// Quantity requested by the failed operation.
    pub requested: Q,
}

impl<R, Q> InsufficientBudgetError<R, Q>
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

    /// Returns the configured finite limit.
    #[must_use]
    #[inline(always)]
    pub const fn limit(&self) -> Q {
        self.limit
    }

    /// Returns the capacity remaining before the failed request.
    #[must_use]
    #[inline(always)]
    pub const fn remaining(&self) -> Q {
        self.remaining
    }

    /// Returns the quantity requested by the failed operation.
    #[must_use]
    #[inline(always)]
    pub const fn requested(&self) -> Q {
        self.requested
    }
}

impl<R, Q> InsufficientBudgetError<R, Q>
where
    Q: crate::ResourceQuantity,
{
    /// Returns the amount already consumed before the failed request.
    #[must_use]
    #[inline]
    pub fn used(&self) -> Q {
        self.limit - self.remaining
    }
}
