// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Structured facts for one exceeded resource limit.

/// Structured facts describing one exceeded resource limit.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LimitExceeded<K> {
    kind: K,
    maximum: usize,
    observed_at_least: usize,
}

impl<K> LimitExceeded<K> {
    /// Creates structured limit-exceeded facts.
    ///
    /// # Parameters
    ///
    /// - `kind`: Domain-specific resource category.
    /// - `maximum`: Largest permitted resource value.
    /// - `observed_at_least`: Saturated value observed by the failed check.
    ///
    /// # Returns
    ///
    /// The structured facts for the exceeded limit.
    #[inline(always)]
    pub const fn new(
        kind: K,
        maximum: usize,
        observed_at_least: usize,
    ) -> Self {
        Self {
            kind,
            maximum,
            observed_at_least,
        }
    }

    /// Returns the domain-specific resource kind.
    ///
    /// # Returns
    ///
    /// A reference to the resource category supplied by the caller.
    #[inline(always)]
    pub const fn kind(&self) -> &K {
        &self.kind
    }

    /// Returns the configured maximum.
    ///
    /// # Returns
    ///
    /// The largest permitted value.
    #[inline(always)]
    pub const fn maximum(&self) -> usize {
        self.maximum
    }

    /// Returns the saturated observed value that exceeded the limit.
    ///
    /// # Returns
    ///
    /// The observed value, saturated at `usize::MAX` when arithmetic
    /// overflowed.
    #[inline(always)]
    pub const fn observed_at_least(&self) -> usize {
        self.observed_at_least
    }

    /// Consumes the facts and returns the resource kind.
    ///
    /// # Returns
    ///
    /// The domain-specific resource category.
    #[inline(always)]
    pub fn into_kind(self) -> K {
        self.kind
    }

    /// Maps the resource kind while preserving the numeric facts.
    ///
    /// # Parameters
    ///
    /// - `mapper`: Function converting the current resource category.
    ///
    /// # Returns
    ///
    /// Limit facts containing the converted category.
    #[inline]
    pub fn map_kind<T>(self, mapper: impl FnOnce(K) -> T) -> LimitExceeded<T> {
        LimitExceeded::new(
            mapper(self.kind),
            self.maximum,
            self.observed_at_least,
        )
    }
}
