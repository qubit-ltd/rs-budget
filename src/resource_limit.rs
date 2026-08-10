// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines immutable point limits bound to resource identities.

use std::fmt::Debug;

use crate::BudgetError;

/// An inclusive immutable maximum for one resource measurement.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained in limit failures.
/// * `Q` - Copyable measurement value used by the maximum and checks.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceLimit<R, Q = u64>
where
    Q: Copy + Debug,
{
    /// Resource bound to this limit.
    resource: R,

    /// Inclusive maximum accepted by this limit.
    maximum: Q,
}

impl<R, Q> ResourceLimit<R, Q>
where
    Q: Copy + Debug,
{
    /// Creates an immutable limit bound to `resource`.
    ///
    /// # Parameters
    ///
    /// * `resource` - Domain resource reported when the limit is exceeded.
    /// * `maximum` - Inclusive maximum measurement accepted by [`Self::check`].
    ///
    /// # Returns
    ///
    /// A limit that accepts measurements less than or equal to `maximum`.
    #[inline]
    pub const fn new(resource: R, maximum: Q) -> Self {
        Self { resource, maximum }
    }

    /// Returns the resource bound to this limit.
    #[inline(always)]
    pub const fn resource(&self) -> &R {
        &self.resource
    }

    /// Returns this limit's inclusive maximum measurement.
    #[inline(always)]
    pub const fn maximum(&self) -> Q {
        self.maximum
    }

    /// Checks whether `actual` is within the inclusive maximum.
    ///
    /// # Parameters
    ///
    /// * `actual` - Observed measurement to validate.
    ///
    /// # Returns
    ///
    /// `Ok(())` when `actual <= maximum`; otherwise returns
    /// [`BudgetError::LimitExceeded`] containing the resource, observed value,
    /// and maximum. This method does not mutate the limit.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::LimitExceeded`] when `actual` is greater than
    /// this limit's maximum.
    #[inline]
    pub fn check(&self, actual: Q) -> Result<(), BudgetError<R, Q>>
    where
        R: Clone,
        Q: PartialOrd,
    {
        if actual > self.maximum {
            Err(BudgetError::LimitExceeded {
                resource: self.resource.clone(),
                actual,
                maximum: self.maximum,
            })
        } else {
            Ok(())
        }
    }
}

/// Checks an optional point limit.
///
/// # Parameters
///
/// * `limit` - Configured limit, or `None` when the dimension is unconfigured.
/// * `actual` - Observed measurement to validate.
///
/// # Returns
///
/// `Ok(())` when `limit` is `None`, or when [`ResourceLimit::check`] accepts
/// `actual`.
///
/// # Errors
///
/// Returns [`BudgetError::LimitExceeded`] when a configured limit rejects
/// `actual`.
#[inline]
pub(crate) fn check_limit<R, Q>(
    limit: Option<&ResourceLimit<R, Q>>,
    actual: Q,
) -> Result<(), BudgetError<R, Q>>
where
    R: Clone,
    Q: Copy + Debug + PartialOrd,
{
    match limit {
        Some(limit) => limit.check(actual),
        None => Ok(()),
    }
}
