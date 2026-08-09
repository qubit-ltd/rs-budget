// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.
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
    pub const fn new(maximum: u64) -> Self {
        Self { maximum }
    }

    /// Returns the finite inclusive maximum.
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
