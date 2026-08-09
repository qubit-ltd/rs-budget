// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors emitted by finite cumulative resource budgets.

use thiserror::Error;

use crate::ResourceQuantity;

/// Facts from a resource consumption request that did not fit.
///
/// The stored `remaining` value is the balance before the failed request.
/// Errors are constructed before any budget mutation, so a failed operation is
/// failure-atomic.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained for diagnostics.
/// * `Q` - Exact unsigned quantity used for the limit and accounting.
#[must_use]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error(
    "resource {resource:?} requested {requested}, but only {remaining} of {limit} remains"
)]
pub struct ResourceBudgetError<R, Q = u64>
where
    Q: ResourceQuantity,
{
    /// Resource value associated with the failed request.
    resource: R,

    /// Finite limit that the request exceeded.
    limit: Q,

    /// Capacity remaining before the failed request.
    remaining: Q,

    /// Quantity requested by the failed operation.
    requested: Q,
}

impl<R, Q> ResourceBudgetError<R, Q>
where
    Q: ResourceQuantity,
{
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
        limit: Q,
        remaining: Q,
        requested: Q,
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
    pub const fn limit(&self) -> Q {
        self.limit
    }

    /// Returns the balance before the failed request.
    #[inline(always)]
    pub const fn remaining(&self) -> Q {
        self.remaining
    }

    /// Returns the requested quantity that did not fit.
    #[inline(always)]
    pub const fn requested(&self) -> Q {
        self.requested
    }

    /// Returns the quantity consumed before the failed request.
    #[inline(always)]
    pub fn used(&self) -> Q {
        self.limit - self.remaining
    }

    /// Returns the exact total that would have been consumed, when
    /// representable.
    ///
    /// `None` means that adding the failed request to the consumed quantity
    /// would overflow the quantity type.
    #[inline(always)]
    pub fn checked_attempted(&self) -> Option<Q> {
        self.used().checked_add(self.requested)
    }
}
