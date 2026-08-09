// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors returned when more capacity is released than was consumed.

/// Facts describing an invalid capacity release.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidRelease {
    releasable: usize,
    requested: usize,
}

impl InvalidRelease {
    /// Creates facts for an invalid release request.
    #[inline(always)]
    pub const fn new(releasable: usize, requested: usize) -> Self {
        Self {
            releasable,
            requested,
        }
    }

    /// Returns the capacity that can be released without exceeding usage.
    #[inline(always)]
    pub const fn releasable(&self) -> usize {
        self.releasable
    }

    /// Returns the amount the caller attempted to release.
    #[inline(always)]
    pub const fn requested(&self) -> usize {
        self.requested
    }
}
