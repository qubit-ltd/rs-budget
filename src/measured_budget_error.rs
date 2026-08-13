// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines failures produced while measuring resource-limited values.
// qubit-style: allow source-test-pair

use std::fmt::Debug;

use thiserror::Error;

use crate::BudgetError;
use crate::QuantityConversionError;

/// Error returned when native measurement or budget validation rejects a value.
#[must_use]
#[derive(Debug, Error)]
pub enum MeasuredBudgetError<R, Q = u64>
where
    Q: Copy + Debug,
{
    /// A native measurement could not fit the configured quantity type.
    #[error(
        "resource {resource:?} has an unrepresentable measurement: {source}"
    )]
    Quantity {
        /// Resource associated with the rejected measurement.
        resource: R,
        /// Native quantity conversion failure.
        #[source]
        source: QuantityConversionError,
    },

    /// A representable measurement exceeded its configured resource budget.
    #[error(transparent)]
    Budget(#[from] BudgetError<R, Q>),
}

impl<R, Q> MeasuredBudgetError<R, Q>
where
    Q: Copy + Debug,
{
    /// Creates a failure for a native measurement that did not fit `Q`.
    ///
    /// # Parameters
    ///
    /// * `resource` - Resource being measured.
    /// * `source` - Exact native quantity conversion failure.
    ///
    /// # Returns
    ///
    /// A quantity representation failure retaining its resource identity.
    #[inline(always)]
    pub const fn quantity(
        resource: R,
        source: QuantityConversionError,
    ) -> Self {
        Self::Quantity { resource, source }
    }

    /// Returns the contained budget failure when the measurement fit `Q`.
    ///
    /// # Returns
    ///
    /// `Some` for [`Self::Budget`], or `None` for [`Self::Quantity`].
    #[must_use]
    #[inline(always)]
    pub const fn budget_error(&self) -> Option<&BudgetError<R, Q>> {
        match self {
            Self::Budget(error) => Some(error),
            Self::Quantity { .. } => None,
        }
    }

    /// Returns the native quantity conversion failure, when present.
    ///
    /// # Returns
    ///
    /// `Some` for [`Self::Quantity`], or `None` for [`Self::Budget`].
    #[must_use]
    #[inline(always)]
    pub const fn quantity_error(&self) -> Option<&QuantityConversionError> {
        match self {
            Self::Quantity { source, .. } => Some(source),
            Self::Budget(_) => None,
        }
    }

    /// Returns the resource associated with this failure.
    ///
    /// The resource is present for both budget validation and quantity
    /// conversion failures, so callers do not need to match the error variant
    /// merely to attach resource context.
    #[must_use]
    #[inline(always)]
    pub const fn resource(&self) -> &R {
        match self {
            Self::Quantity { resource, .. } => resource,
            Self::Budget(error) => error.resource(),
        }
    }

    /// Consumes this failure and returns its associated resource.
    #[inline(always)]
    pub fn into_resource(self) -> R {
        match self {
            Self::Quantity { resource, .. } => resource,
            Self::Budget(error) => error.into_resource(),
        }
    }
}
