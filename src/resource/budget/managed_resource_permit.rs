// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines an RAII permit returned by a managed resource pool.

use std::sync::Arc;

use super::internal::ManagedResourcePoolInner;
use crate::resource::ResourceQuantity;

/// Owns acquired capacity and returns it to its pool when dropped.
///
/// A permit may move across threads when its resource and quantity types allow
/// that movement. Calling [`Self::release`] returns capacity early; otherwise
/// normal Drop, early returns, `?`, and panic unwinding all return it.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource value retained by the owning pool.
/// * `Q` - Exact unsigned quantity owned by this permit.
///
/// # Examples
///
/// ```
/// use qubit_budget::ManagedResourcePool;
///
/// let pool = ManagedResourcePool::new("connections", 1_u64);
/// let permit = pool.try_acquire(1).expect("one connection should fit");
/// assert_eq!(permit.amount(), 1);
/// permit.release();
/// assert_eq!(pool.available(), 1);
/// ```
#[must_use = "dropping the permit releases its acquired capacity"]
#[derive(Debug)]
pub struct ManagedResourcePermit<R, Q = u64>
where
    Q: ResourceQuantity,
{
    /// Shared state receiving this permit's quantity on release.
    inner: Option<Arc<ManagedResourcePoolInner<R, Q>>>,
    /// Quantity uniquely owned by this permit.
    amount: Q,
}

impl<R, Q> ManagedResourcePermit<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates a permit for capacity already deducted from `inner`.
    #[inline]
    pub(super) fn new(inner: Arc<ManagedResourcePoolInner<R, Q>>, amount: Q) -> Self {
        Self {
            inner: Some(inner),
            amount,
        }
    }

    /// Returns the resource whose capacity this permit owns.
    ///
    /// # Panics
    ///
    /// Panics only if an internal invariant is violated and a live permit no
    /// longer retains its owning pool.
    #[must_use]
    #[inline(always)]
    pub fn resource(&self) -> &R {
        self.inner
            .as_ref()
            .expect("a live managed resource permit always retains its pool")
            .limit
            .resource()
    }

    /// Returns the quantity owned by this permit.
    #[must_use]
    #[inline(always)]
    pub const fn amount(&self) -> Q {
        self.amount
    }

    /// Returns this permit's capacity before the end of its lexical scope.
    ///
    /// Consuming `self` prevents callers from releasing the same permit twice.
    pub fn release(mut self) {
        self.release_inner();
    }

    /// Returns capacity once and leaves Drop with no remaining work.
    #[inline]
    fn release_inner(&mut self) {
        if let Some(inner) = self.inner.take() {
            inner.release(self.amount);
        }
    }
}

impl<R, Q> Drop for ManagedResourcePermit<R, Q>
where
    Q: ResourceQuantity,
{
    /// Returns owned capacity without panicking during unwinding.
    fn drop(&mut self) {
        self.release_inner();
    }
}
