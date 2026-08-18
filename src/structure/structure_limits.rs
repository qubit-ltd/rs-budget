// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines optional structural input limits.

use super::StructureBudget;
use super::StructureLimitsBuilder;
use super::StructureResource;
use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;

/// Optional limits for processing nested structural data.
///
/// `R` identifies the resource values reported in [`crate::BudgetError`], and
/// `Q` is the exact unsigned quantity used for all measurements. The default
/// configuration uses [`StructureResource`] and [`usize`].
///
/// # Examples
///
/// ```
/// use qubit_budget::StructureLimits;
///
/// let limits = StructureLimits::builder()
///     .max_depth(4)
///     .max_nodes(16)
///     .build();
/// let mut budget = limits.budget();
///
/// budget.check_depth(4).expect("the inclusive depth limit should fit");
/// budget.charge_node().expect("the first node should fit");
/// assert_eq!(budget.used_nodes(), 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructureLimits<R = StructureResource, Q = usize>
where
    Q: ResourceQuantity,
{
    /// Optional inclusive maximum nesting depth.
    max_depth: Option<ResourceLimit<R, Q>>,

    /// Optional cumulative maximum number of processed nodes.
    max_nodes: Option<ResourceLimit<R, Q>>,

    /// Optional inclusive maximum number of items in one sequence.
    max_sequence_items: Option<ResourceLimit<R, Q>>,

    /// Optional inclusive maximum number of entries in one map.
    max_map_entries: Option<ResourceLimit<R, Q>>,

    /// Optional inclusive maximum byte length of one structural key.
    max_key_bytes: Option<ResourceLimit<R, Q>>,
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

    /// Converts these limits into a builder for further configuration.
    #[inline]
    #[must_use]
    pub const fn into_builder(self) -> StructureLimitsBuilder<R, Q> {
        StructureLimitsBuilder::from_limits(self)
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

    /// Replaces the depth limit during builder composition.
    #[inline(always)]
    pub(super) fn set_depth_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.max_depth = Some(limit);
    }

    /// Replaces the node limit during builder composition.
    #[inline(always)]
    pub(super) fn set_nodes_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.max_nodes = Some(limit);
    }

    /// Replaces the sequence-item limit during builder composition.
    #[inline(always)]
    pub(super) fn set_sequence_items_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.max_sequence_items = Some(limit);
    }

    /// Replaces the map-entry limit during builder composition.
    #[inline(always)]
    pub(super) fn set_map_entries_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.max_map_entries = Some(limit);
    }

    /// Replaces the key-byte limit during builder composition.
    #[inline(always)]
    pub(super) fn set_key_bytes_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.max_key_bytes = Some(limit);
    }
}

impl StructureLimits<StructureResource, usize> {
    /// Replaces the standard depth limit in a const builder operation.
    #[inline(always)]
    pub(super) const fn set_max_depth(&mut self, maximum: usize) {
        self.max_depth = Some(ResourceLimit::new(StructureResource::Depth, maximum));
    }

    /// Replaces the standard node limit in a const builder operation.
    #[inline(always)]
    pub(super) const fn set_max_nodes(&mut self, maximum: usize) {
        self.max_nodes = Some(ResourceLimit::new(StructureResource::Nodes, maximum));
    }

    /// Replaces the standard sequence-item limit in a const builder operation.
    #[inline(always)]
    pub(super) const fn set_max_sequence_items(&mut self, maximum: usize) {
        self.max_sequence_items = Some(ResourceLimit::new(StructureResource::SequenceItems, maximum));
    }

    /// Replaces the standard map-entry limit in a const builder operation.
    #[inline(always)]
    pub(super) const fn set_max_map_entries(&mut self, maximum: usize) {
        self.max_map_entries = Some(ResourceLimit::new(StructureResource::MapEntries, maximum));
    }

    /// Replaces the standard key-byte limit in a const builder operation.
    #[inline(always)]
    pub(super) const fn set_max_key_bytes(&mut self, maximum: usize) {
        self.max_key_bytes = Some(ResourceLimit::new(StructureResource::KeyBytes, maximum));
    }
}
