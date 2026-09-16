// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the synchronized state shared by one managed resource pool.

use std::sync::Mutex;
use std::sync::MutexGuard;

use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;

/// Stores one finite limit and its synchronized available capacity.
#[derive(Debug)]
pub(in crate::resource::budget) struct ManagedResourcePoolInner<R, Q>
where
    Q: ResourceQuantity,
{
    /// Immutable resource limit shared by all handles and permits.
    pub(in crate::resource::budget) limit: ResourceLimit<R, Q>,
    /// Capacity that has not currently been acquired.
    available: Mutex<Q>,
}

impl<R, Q> ManagedResourcePoolInner<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates synchronized state with all finite capacity available.
    ///
    /// # Parameters
    ///
    /// * `limit` - Immutable resource identity and capacity.
    ///
    /// # Returns
    ///
    /// New shared state whose availability equals the configured maximum.
    #[must_use]
    #[inline]
    pub(in crate::resource::budget) fn new(limit: ResourceLimit<R, Q>) -> Self {
        let available = limit.maximum();
        Self {
            limit,
            available: Mutex::new(available),
        }
    }

    /// Locks the available quantity, recovering primitive state after poison.
    ///
    /// # Returns
    ///
    /// A guard for the synchronized available quantity. A poisoned lock is
    /// recovered because the protected critical sections only update `Q`.
    #[inline]
    pub(in crate::resource::budget) fn lock_available(&self) -> MutexGuard<'_, Q> {
        self.available.lock().unwrap_or_else(|error| error.into_inner())
    }

    /// Returns acquired capacity to this state without allowing Drop to panic.
    ///
    /// # Parameters
    ///
    /// * `amount` - Quantity previously acquired by exactly one permit.
    ///
    /// If an internal invariant is violated, availability is defensively
    /// capped at the configured capacity rather than panicking in a destructor.
    pub(in crate::resource::budget) fn release(&self, amount: Q) {
        let capacity = self.limit.maximum();
        let mut available = self.lock_available();
        *available = match available.checked_add(amount) {
            Some(next) if next <= capacity => next,
            _ => capacity,
        };
    }
}
