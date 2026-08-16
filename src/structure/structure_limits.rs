// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow multiple-public-types
//! Defines optional structural input limits.

use crate::ResourceLimit;
use crate::ResourceQuantity;
use crate::StructureBudget;
use crate::StructureResource;

/// Optional limits for processing nested structural data.
///
/// `R` identifies the resource values reported in [`crate::BudgetError`], and
/// `Q` is the exact unsigned quantity used for all measurements. The default
/// configuration uses [`StructureResource`] and [`usize`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructureLimits<R = StructureResource, Q = usize>
where
    Q: ResourceQuantity,
{
    /// Optional inclusive maximum nesting depth.
    pub(crate) max_depth: Option<ResourceLimit<R, Q>>,

    /// Optional cumulative maximum number of processed nodes.
    pub(crate) max_nodes: Option<ResourceLimit<R, Q>>,

    /// Optional inclusive maximum number of items in one sequence.
    pub(crate) max_sequence_items: Option<ResourceLimit<R, Q>>,

    /// Optional inclusive maximum number of entries in one map.
    pub(crate) max_map_entries: Option<ResourceLimit<R, Q>>,

    /// Optional inclusive maximum byte length of one structural key.
    pub(crate) max_key_bytes: Option<ResourceLimit<R, Q>>,
}

impl<R, Q> Default for StructureLimits<R, Q>
where
    Q: ResourceQuantity,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<R, Q> StructureLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an unconfigured custom-resource limit set.
    ///
    /// The default [`StructureResource`]/`usize` configuration is constructed
    /// with [`Self::new`]. Custom resource sets use this constructor because
    /// their resource identity is supplied by each `*_limit` method.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_depth: None,
            max_nodes: None,
            max_sequence_items: None,
            max_map_entries: None,
            max_key_bytes: None,
        }
    }

    /// Creates a builder for structural limits.
    #[inline]
    #[must_use]
    pub const fn builder() -> StructureLimitsBuilder<R, Q> {
        StructureLimitsBuilder::new()
    }

    /// Returns the complete depth limit, when configured.
    #[must_use]
    #[inline(always)]
    pub const fn depth_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_depth.as_ref()
    }

    /// Returns the complete node limit, when configured.
    #[must_use]
    #[inline(always)]
    pub const fn nodes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_nodes.as_ref()
    }

    /// Returns the complete sequence-item limit, when configured.
    #[must_use]
    #[inline(always)]
    pub const fn sequence_items_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_sequence_items.as_ref()
    }

    /// Returns the complete map-entry limit, when configured.
    #[must_use]
    #[inline(always)]
    pub const fn map_entries_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_map_entries.as_ref()
    }

    /// Returns the complete structural-key limit, when configured.
    #[must_use]
    #[inline(always)]
    pub const fn key_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_key_bytes.as_ref()
    }

    /// Returns the configured maximum nesting depth.
    #[must_use]
    #[inline(always)]
    pub const fn max_depth(&self) -> Option<Q> {
        match self.max_depth.as_ref() {
            Some(limit) => Some(limit.maximum()),
            None => None,
        }
    }

    /// Returns the configured maximum number of processed nodes.
    #[must_use]
    #[inline(always)]
    pub const fn max_nodes(&self) -> Option<Q> {
        match self.max_nodes.as_ref() {
            Some(limit) => Some(limit.maximum()),
            None => None,
        }
    }

    /// Returns the configured maximum number of items in one sequence.
    #[must_use]
    #[inline(always)]
    pub const fn max_sequence_items(&self) -> Option<Q> {
        match self.max_sequence_items.as_ref() {
            Some(limit) => Some(limit.maximum()),
            None => None,
        }
    }

    /// Returns the configured maximum number of entries in one map.
    #[must_use]
    #[inline(always)]
    pub const fn max_map_entries(&self) -> Option<Q> {
        match self.max_map_entries.as_ref() {
            Some(limit) => Some(limit.maximum()),
            None => None,
        }
    }

    /// Returns the configured maximum byte length of one structural key.
    #[must_use]
    #[inline(always)]
    pub const fn max_key_bytes(&self) -> Option<Q> {
        match self.max_key_bytes.as_ref() {
            Some(limit) => Some(limit.maximum()),
            None => None,
        }
    }

    /// Creates an independent structural budget session from these limits.
    #[inline]
    #[must_use]
    pub fn budget(&self) -> StructureBudget<R, Q>
    where
        R: Clone,
    {
        StructureBudget::new(self.clone())
    }
}

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

    /// Sets the depth limit.
    #[inline]
    #[must_use]
    pub fn depth_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.max_depth = Some(limit);
        self
    }

    /// Sets the node limit.
    #[inline]
    #[must_use]
    pub fn nodes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.max_nodes = Some(limit);
        self
    }

    /// Sets the sequence-item limit.
    #[inline]
    #[must_use]
    pub fn sequence_items_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.max_sequence_items = Some(limit);
        self
    }

    /// Sets the map-entry limit.
    #[inline]
    #[must_use]
    pub fn map_entries_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.max_map_entries = Some(limit);
        self
    }

    /// Sets the structural-key limit.
    #[inline]
    #[must_use]
    pub fn key_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.max_key_bytes = Some(limit);
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
        self.limits.max_depth =
            Some(ResourceLimit::new(StructureResource::Depth, maximum));
        self
    }

    /// Sets the maximum number of processed nodes.
    #[inline]
    #[must_use]
    pub const fn max_nodes(mut self, maximum: usize) -> Self {
        self.limits.max_nodes =
            Some(ResourceLimit::new(StructureResource::Nodes, maximum));
        self
    }

    /// Sets the maximum number of items in one sequence.
    #[inline]
    #[must_use]
    pub const fn max_sequence_items(mut self, maximum: usize) -> Self {
        self.limits.max_sequence_items = Some(ResourceLimit::new(
            StructureResource::SequenceItems,
            maximum,
        ));
        self
    }

    /// Sets the maximum number of entries in one map.
    #[inline]
    #[must_use]
    pub const fn max_map_entries(mut self, maximum: usize) -> Self {
        self.limits.max_map_entries =
            Some(ResourceLimit::new(StructureResource::MapEntries, maximum));
        self
    }

    /// Sets the maximum byte length of one structural key.
    #[inline]
    #[must_use]
    pub const fn max_key_bytes(mut self, maximum: usize) -> Self {
        self.limits.max_key_bytes =
            Some(ResourceLimit::new(StructureResource::KeyBytes, maximum));
        self
    }
}
