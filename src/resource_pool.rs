// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
// =============================================================================
//! Defines finite releasable resource pools.

use crate::ResourceLimit;
use crate::ResourcePoolError;

/// A finite, non-synchronizing pool of releasable resource capacity.
///
/// Acquisition subtracts from `available`; release adds only after checking
/// the amount is no greater than `in_use`. The object has no lifecycle state,
/// waiting, fairness, permits or cancellation. An unconfigured dimension is
/// represented by `Option<ResourcePool<R>> = None`.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourcePool<R> {
    resource: R,
    limit: ResourceLimit,
    available: u64,
}

impl<R> ResourcePool<R> {
    /// Creates an entirely available finite pool.
    ///
    /// # Parameters
    ///
    /// * `resource` - Domain resource value retained in errors.
    /// * `limit` - Finite pool capacity.
    ///
    /// # Returns
    ///
    /// A pool with `available == limit.maximum()`.
    pub fn new(resource: R, limit: ResourceLimit) -> Self {
        Self {
            resource,
            limit,
            available: limit.maximum(),
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
    pub fn try_acquire(
        &mut self,
        amount: u64,
    ) -> Result<(), ResourcePoolError<R>>
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
        self.available -= amount;
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
    pub fn release(&mut self, amount: u64) -> Result<(), ResourcePoolError<R>>
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
        self.available += amount;
        Ok(())
    }

    /// Returns the associated resource.
    pub const fn resource(&self) -> &R {
        &self.resource
    }

    /// Returns the finite pool limit.
    pub const fn limit(&self) -> ResourceLimit {
        self.limit
    }

    /// Returns the total finite capacity.
    pub const fn capacity(&self) -> u64 {
        self.limit.maximum()
    }

    /// Returns currently available capacity.
    pub const fn available(&self) -> u64 {
        self.available
    }

    /// Returns currently acquired capacity.
    pub const fn in_use(&self) -> u64 {
        self.limit.maximum() - self.available
    }
}
