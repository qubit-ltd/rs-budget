// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines finite releasable resource pools.

use crate::ResourcePoolError;
use crate::ResourceQuantity;

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
#[must_use]
#[derive(Debug, PartialEq, Eq)]
pub struct ResourcePool<R, Q = u64>
where
    Q: ResourceQuantity,
{
    /// Resource value retained in acquisition and release errors.
    resource: R,

    /// Finite total capacity of the pool.
    limit: Q,

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
    pub const fn new(resource: R, limit: Q) -> Self {
        Self {
            resource,
            limit,
            available: limit,
        }
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
    /// [`ResourcePoolError::Exhausted`] with no state change when it does
    /// not fit.
    ///
    /// # Errors
    ///
    /// Returns [`ResourcePoolError::Exhausted`] when `amount` exceeds current
    /// availability. The pool remains unchanged in that case.
    pub fn try_acquire(
        &mut self,
        amount: Q,
    ) -> Result<(), ResourcePoolError<R, Q>>
    where
        R: Clone,
    {
        if amount > self.available {
            return Err(ResourcePoolError::Exhausted {
                resource: self.resource.clone(),
                limit: self.limit,
                available: self.available,
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
    /// [`ResourcePoolError::InvalidRelease`] with no state change when the
    /// amount exceeds current occupancy.
    ///
    /// # Errors
    ///
    /// Returns [`ResourcePoolError::InvalidRelease`] when `amount` exceeds
    /// current occupancy. The pool remains unchanged in that case.
    pub fn release(&mut self, amount: Q) -> Result<(), ResourcePoolError<R, Q>>
    where
        R: Clone,
    {
        let in_use = self.in_use();
        if amount > in_use {
            return Err(ResourcePoolError::InvalidRelease {
                resource: self.resource.clone(),
                limit: self.limit,
                in_use,
                requested: amount,
            });
        }
        self.available = self.available + amount;
        Ok(())
    }

    /// Returns the associated resource.
    #[inline(always)]
    pub const fn resource(&self) -> &R {
        &self.resource
    }

    /// Returns the finite pool limit.
    #[inline(always)]
    pub const fn limit(&self) -> Q {
        self.limit
    }

    /// Returns the total finite capacity.
    #[inline(always)]
    pub const fn capacity(&self) -> Q {
        self.limit
    }

    /// Returns currently available capacity.
    #[inline(always)]
    pub const fn available(&self) -> Q {
        self.available
    }

    /// Returns currently acquired capacity.
    #[inline(always)]
    pub fn in_use(&self) -> Q {
        self.limit - self.available
    }
}
