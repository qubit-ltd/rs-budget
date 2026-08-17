// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Builds optional structural input limits.

use super::StructureLimits;
use super::StructureResource;
use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;

/// Builder for [`StructureLimits`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructureLimitsBuilder<R = StructureResource, Q = usize>
where
    Q: ResourceQuantity,
{
    limits: StructureLimits<R, Q>,
}

impl<R, Q> Default for StructureLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<R, Q> From<StructureLimitsBuilder<R, Q>> for StructureLimits<R, Q>
where
    Q: ResourceQuantity,
{
    fn from(builder: StructureLimitsBuilder<R, Q>) -> Self {
        builder.build()
    }
}

impl<R, Q> StructureLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an empty structural-limits builder.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            limits: StructureLimits::new(),
        }
    }

    /// Creates a builder retaining an existing limit configuration.
    #[inline]
    #[must_use]
    pub(crate) const fn from_limits(limits: StructureLimits<R, Q>) -> Self {
        Self { limits }
    }

    /// Sets the depth limit.
    #[inline]
    #[must_use]
    pub fn depth_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_depth_limit(limit);
        self
    }

    /// Sets the node limit.
    #[inline]
    #[must_use]
    pub fn nodes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_nodes_limit(limit);
        self
    }

    /// Sets the sequence-item limit.
    #[inline]
    #[must_use]
    pub fn sequence_items_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_sequence_items_limit(limit);
        self
    }

    /// Sets the map-entry limit.
    #[inline]
    #[must_use]
    pub fn map_entries_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_map_entries_limit(limit);
        self
    }

    /// Sets the structural-key limit.
    #[inline]
    #[must_use]
    pub fn key_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_key_bytes_limit(limit);
        self
    }

    /// Builds the configured structural limits.
    #[inline]
    #[must_use]
    pub fn build(self) -> StructureLimits<R, Q> {
        self.limits
    }
}

impl StructureLimitsBuilder<StructureResource, usize> {
    /// Sets the maximum nesting depth.
    #[inline]
    #[must_use]
    pub const fn max_depth(mut self, maximum: usize) -> Self {
        self.limits.set_max_depth(maximum);
        self
    }

    /// Sets the maximum number of processed nodes.
    #[inline]
    #[must_use]
    pub const fn max_nodes(mut self, maximum: usize) -> Self {
        self.limits.set_max_nodes(maximum);
        self
    }

    /// Sets the maximum number of items in one sequence.
    #[inline]
    #[must_use]
    pub const fn max_sequence_items(mut self, maximum: usize) -> Self {
        self.limits.set_max_sequence_items(maximum);
        self
    }

    /// Sets the maximum number of entries in one map.
    #[inline]
    #[must_use]
    pub const fn max_map_entries(mut self, maximum: usize) -> Self {
        self.limits.set_max_map_entries(maximum);
        self
    }

    /// Sets the maximum byte length of one structural key.
    #[inline]
    #[must_use]
    pub const fn max_key_bytes(mut self, maximum: usize) -> Self {
        self.limits.set_max_key_bytes(maximum);
        self
    }
}
