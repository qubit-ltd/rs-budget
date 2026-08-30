// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines a cloneable finite pool that returns capacity through RAII permits.

use std::sync::Arc;

use super::ManagedResourcePermit;
use super::internal::ManagedResourcePoolInner;
use crate::resource::InsufficientBudgetError;
use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;

/// A cloneable finite pool whose acquired capacity is owned by RAII permits.
///
/// Clones share one synchronized capacity state. Dropping a returned
/// [`ManagedResourcePermit`] makes its quantity available again, including
/// during early returns and panic unwinding. Use [`crate::ResourcePool`] when
/// explicit `try_acquire`/`release` pairing is preferred.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained for diagnostics.
/// * `Q` - Exact unsigned quantity shared by the pool and its permits.
///
/// # Examples
///
/// ```
/// use qubit_budget::ManagedResourcePool;
///
/// let pool = ManagedResourcePool::new("workers", 2_u64);
/// let permit = pool.try_acquire(1).expect("one worker should fit");
/// assert_eq!(pool.in_use(), 1);
/// drop(permit);
/// assert_eq!(pool.available(), 2);
/// ```
#[derive(Debug)]
pub struct ManagedResourcePool<R, Q = u64>
where
    Q: ResourceQuantity,
{
    /// Synchronized capacity shared by every handle and outstanding permit.
    inner: Arc<ManagedResourcePoolInner<R, Q>>,
}

impl<R, Q> Clone for ManagedResourcePool<R, Q>
where
    Q: ResourceQuantity,
{
    /// Clones the shared handle without duplicating finite capacity.
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<R, Q> ManagedResourcePool<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates a managed pool with all finite capacity available.
    ///
    /// # Parameters
    ///
    /// * `resource` - Resource identity retained in acquisition errors.
    /// * `limit` - Total finite capacity shared by all handles.
    ///
    /// # Returns
    ///
    /// A new managed pool with `limit` units available.
    #[must_use]
    #[inline]
    pub fn new(resource: R, limit: Q) -> Self {
        Self::from_limit(ResourceLimit::new(resource, limit))
    }

    /// Creates a managed pool from an immutable resource limit.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource identity and total finite capacity.
    ///
    /// # Returns
    ///
    /// A new managed pool preserving the supplied limit.
    #[must_use]
    #[inline]
    pub fn from_limit(limit: ResourceLimit<R, Q>) -> Self {
        Self {
            inner: Arc::new(ManagedResourcePoolInner::new(limit)),
        }
    }

    /// Acquires capacity and returns a permit that releases it on Drop.
    ///
    /// # Parameters
    ///
    /// * `amount` - Quantity to acquire from current availability.
    ///
    /// # Returns
    ///
    /// A permit owning `amount` units when they fit.
    ///
    /// # Errors
    ///
    /// Returns [`InsufficientBudgetError`] when `amount` exceeds current
    /// availability. Failure leaves the shared pool unchanged. The resource is
    /// cloned only after releasing the internal lock.
    pub fn try_acquire(&self, amount: Q) -> Result<ManagedResourcePermit<R, Q>, InsufficientBudgetError<R, Q>>
    where
        R: Clone,
    {
        let remaining = {
            let mut available = self.inner.lock_available();
            if amount <= *available {
                *available = *available - amount;
                return Ok(ManagedResourcePermit::new(Arc::clone(&self.inner), amount));
            }
            *available
        };
        Err(InsufficientBudgetError {
            resource: self.resource().clone(),
            limit: self.capacity(),
            remaining,
            requested: amount,
        })
    }

    /// Returns the resource associated with this shared pool.
    #[must_use]
    #[inline(always)]
    pub fn resource(&self) -> &R {
        self.inner.limit.resource()
    }

    /// Returns the immutable resource limit configuring this pool.
    #[must_use]
    #[inline(always)]
    pub fn resource_limit(&self) -> &ResourceLimit<R, Q> {
        &self.inner.limit
    }

    /// Returns total finite capacity.
    #[must_use]
    #[inline(always)]
    pub fn capacity(&self) -> Q {
        self.inner.limit.maximum()
    }

    /// Returns capacity not currently owned by permits.
    #[must_use]
    pub fn available(&self) -> Q {
        *self.inner.lock_available()
    }

    /// Returns capacity currently owned by permits.
    #[must_use]
    #[inline]
    pub fn in_use(&self) -> Q {
        self.capacity() - self.available()
    }
}
