// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines finite limits for one resource observation.

use crate::LimitExceeded;

/// An immutable finite inclusive maximum for a resource quantity.
///
/// The limit itself is deliberately independent of a resource value. Pass the
/// resource to [`Self::check`] when an exceeded observation needs structured
/// diagnostic context. An unconfigured limit is represented by the caller as
/// `Option::None`; this type has no unlimited variant.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceLimit {
    /// Largest permitted resource quantity.
    maximum: u64,
}

impl ResourceLimit {
    /// Creates a finite inclusive limit.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Largest permitted resource quantity.
    ///
    /// # Returns
    ///
    /// A limit that accepts observations from zero through `maximum`.
    #[inline]
    pub const fn new(maximum: u64) -> Self {
        Self { maximum }
    }

    /// Returns the finite inclusive maximum.
    #[inline(always)]
    pub const fn maximum(&self) -> u64 {
        self.maximum
    }

    /// Checks one observed resource quantity.
    ///
    /// # Parameters
    ///
    /// * `resource` - Domain resource value retained in an exceeded error.
    /// * `observed` - Quantity to compare with this limit.
    ///
    /// # Returns
    ///
    /// `Ok(())` when `observed <= maximum`; otherwise returns exact facts in
    /// [`LimitExceeded`]. This method has no mutable state or side effects.
    ///
    /// # Errors
    ///
    /// Returns [`LimitExceeded`] when `observed` is greater than this limit.
    ///
    /// # Type Parameters
    ///
    /// * `R` - Caller-defined resource value retained in the error.
    pub fn check<R>(
        &self,
        resource: R,
        observed: u64,
    ) -> Result<(), LimitExceeded<R>> {
        if observed <= self.maximum {
            Ok(())
        } else {
            Err(LimitExceeded::new(resource, *self, observed))
        }
    }
}
