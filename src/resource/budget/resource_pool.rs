// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines finite releasable resource pools.

use crate::resource::InsufficientBudgetError;
use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;
use crate::resource::ResourceReleaseError;

/// A finite, non-synchronizing pool of releasable resource capacity.
///
/// Acquisition subtracts from `available`; release adds only after checking
/// the amount is no greater than `in_use`. The object has no lifecycle state,
/// waiting, fairness, permits or cancellation. An unconfigured dimension is
/// represented by `Option<ResourcePool<R>> = None`.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained for diagnostics.
/// * `Q` - Exact unsigned quantity used for the capacity and accounting.
///
/// # Examples
///
/// ```
/// use qubit_budget::ResourcePool;
///
/// let mut pool = ResourcePool::new("workers", 2_u64);
/// pool.try_acquire(1).expect("one worker should fit");
/// assert_eq!(pool.in_use(), 1);
/// pool.release(1).expect("the worker is returned");
/// assert_eq!(pool.available(), 2);
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct ResourcePool<R, Q = u64>
where
    Q: ResourceQuantity,
{
    /// Finite total capacity of the pool.
    limit: ResourceLimit<R, Q>,

    /// Capacity that is currently available for acquisition.
    available: Q,
}

impl<R, Q> ResourcePool<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an entirely available finite pool.
    ///
    /// # Parameters
    ///
    /// * `resource` - Domain resource value retained in errors.
    /// * `limit` - Finite pool capacity.
    ///
    /// # Returns
    ///
    /// A pool with `available == limit`.
    ///
    /// # Examples
    ///
    /// ```
    /// use qubit_budget::ResourcePool;
    ///
    /// let mut pool = ResourcePool::new("readers", 2_u64);
    /// pool.try_acquire(1).expect("one reader should fit");
    /// pool.release(1).expect("the reader is returned explicitly");
    /// assert_eq!(pool.in_use(), 0);
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(resource: R, limit: Q) -> Self {
        Self {
            limit: ResourceLimit::new(resource, limit),
            available: limit,
        }
    }

    /// Creates an entirely available pool from an immutable resource limit.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource identity and finite capacity for this pool.
    ///
    /// # Returns
    ///
    /// A pool whose available capacity equals the limit maximum.
    #[inline]
    #[must_use]
    pub fn from_limit(limit: ResourceLimit<R, Q>) -> Self {
        let available = limit.maximum();
        Self { limit, available }
    }

    /// Acquires capacity when enough units are available.
    ///
    /// # Parameters
    ///
    /// * `amount` - Quantity to acquire.
    ///
    /// # Returns
    ///
    /// `Ok(())` after subtracting the amount, or
    /// [`InsufficientBudgetError`] with no state change when it does
    /// not fit.
    ///
    /// # Errors
    ///
    /// Returns [`InsufficientBudgetError`] when `amount` exceeds current
    /// availability. The pool remains unchanged in that case.
    pub fn try_acquire(&mut self, amount: Q) -> Result<(), InsufficientBudgetError<R, Q>>
    where
        R: Clone,
    {
        if amount > self.available {
            return Err(InsufficientBudgetError {
                resource: self.limit.resource().clone(),
                limit: self.limit.maximum(),
                remaining: self.available,
                requested: amount,
            });
        }
        self.available = self.available - amount;
        Ok(())
    }

    /// Releases previously acquired capacity.
    ///
    /// # Parameters
    ///
    /// * `amount` - Quantity to return to the pool.
    ///
    /// # Returns
    ///
    /// `Ok(())` after increasing availability, or
    /// [`ResourceReleaseError`] with no state change when the
    /// amount exceeds current occupancy.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceReleaseError`] when `amount` exceeds
    /// current occupancy. The pool remains unchanged in that case.
    pub fn release(&mut self, amount: Q) -> Result<(), ResourceReleaseError<R, Q>>
    where
        R: Clone,
    {
        let in_use = self.in_use();
        if amount > in_use {
            return Err(ResourceReleaseError::InvalidRelease {
                resource: self.limit.resource().clone(),
                limit: self.limit.maximum(),
                in_use,
                requested: amount,
            });
        }
        self.available = self.available + amount;
        Ok(())
    }

    /// Returns the associated resource.
    ///
    /// # Returns
    ///
    /// Returns the associated resource.
    #[must_use]
    #[inline(always)]
    pub const fn resource(&self) -> &R {
        self.limit.resource()
    }

    /// Returns the immutable resource limit that configures this pool.
    ///
    /// # Returns
    ///
    /// Returns the immutable resource limit that configures this pool.
    #[must_use]
    #[inline(always)]
    pub const fn resource_limit(&self) -> &ResourceLimit<R, Q> {
        &self.limit
    }

    /// Returns the total finite capacity.
    ///
    /// # Returns
    ///
    /// Returns the total finite capacity.
    #[must_use]
    #[inline(always)]
    pub const fn capacity(&self) -> Q {
        self.limit.maximum()
    }

    /// Returns currently available capacity.
    ///
    /// # Returns
    ///
    /// Returns currently available capacity.
    #[must_use]
    #[inline(always)]
    pub const fn available(&self) -> Q {
        self.available
    }

    /// Returns currently acquired capacity.
    ///
    /// # Returns
    ///
    /// Returns currently acquired capacity.
    #[inline(always)]
    #[must_use]
    pub fn in_use(&self) -> Q {
        self.limit.maximum() - self.available
    }
}
