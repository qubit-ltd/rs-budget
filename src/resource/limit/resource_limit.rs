// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines immutable point limits bound to resource identities.

use std::fmt::Debug;

use crate::resource::BudgetError;
use crate::resource::LimitExceededError;
use crate::resource::MeasuredBudgetError;
use crate::resource::Observation;
use crate::resource::ResourceQuantity;

/// An inclusive immutable maximum for one resource measurement.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained in limit failures.
/// * `Q` - Copyable measurement value used by the maximum and checks.
///
/// # Examples
///
/// ```
/// use qubit_budget::ResourceLimit;
///
/// let limit = ResourceLimit::new("payload bytes", 8_u64);
/// limit.check(8).expect("the inclusive maximum should fit");
/// assert!(limit.check(9).is_err());
/// ```
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
    #[must_use]
    pub const fn new(resource: R, maximum: Q) -> Self {
        Self { resource, maximum }
    }

    /// Returns the resource bound to this limit.
    #[inline(always)]
    #[must_use]
    pub const fn resource(&self) -> &R {
        &self.resource
    }

    /// Returns this limit's inclusive maximum measurement.
    #[inline(always)]
    #[must_use]
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
    /// [`LimitExceededError`] containing the resource, observed value,
    /// and maximum. This method does not mutate the limit.
    ///
    /// # Errors
    ///
    /// Returns [`LimitExceededError`] when `actual` is greater than
    /// this limit's maximum.
    #[inline]
    pub fn check(&self, actual: Q) -> Result<(), LimitExceededError<R, Q>>
    where
        R: Clone,
        Q: Ord,
    {
        if actual > self.maximum {
            Err(LimitExceededError {
                resource: self.resource.clone(),
                observed: Observation::Exact(actual),
                maximum: self.maximum,
            })
        } else {
            Ok(())
        }
    }
}

impl<R, Q> ResourceLimit<R, Q>
where
    Q: ResourceQuantity,
{
    /// Checks a machine-sized measurement without truncating it.
    ///
    /// # Parameters
    ///
    /// * `actual` - Native measurement to convert and compare with the limit.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the converted measurement fits this limit.
    ///
    /// # Errors
    ///
    /// Returns [`MeasuredBudgetError::Quantity`] when `actual` cannot be
    /// represented by `Q`, or [`MeasuredBudgetError::Budget`] when the
    /// converted value exceeds this limit.
    #[inline]
    pub fn check_usize(
        &self,
        actual: usize,
    ) -> Result<(), MeasuredBudgetError<R, Q>>
    where
        R: Clone,
    {
        let actual = Q::try_from_usize(actual).map_err(|source| {
            MeasuredBudgetError::quantity(self.resource.clone(), source)
        })?;
        self.check(actual).map_err(MeasuredBudgetError::from)
    }

    /// Checks a 64-bit measurement without truncating it.
    ///
    /// # Parameters
    ///
    /// * `actual` - 64-bit measurement to convert and compare with the limit.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the converted measurement fits this limit.
    ///
    /// # Errors
    ///
    /// Returns [`MeasuredBudgetError::Quantity`] when `actual` cannot be
    /// represented by `Q`, or [`MeasuredBudgetError::Budget`] when the
    /// converted value exceeds this limit.
    #[inline]
    pub fn check_u64(
        &self,
        actual: u64,
    ) -> Result<(), MeasuredBudgetError<R, Q>>
    where
        R: Clone,
    {
        let actual = Q::try_from_u64(actual).map_err(|source| {
            MeasuredBudgetError::quantity(self.resource.clone(), source)
        })?;
        self.check(actual).map_err(MeasuredBudgetError::from)
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
    Q: Copy + Debug + Ord,
{
    match limit {
        Some(limit) => limit.check(actual).map_err(BudgetError::from),
        None => Ok(()),
    }
}
