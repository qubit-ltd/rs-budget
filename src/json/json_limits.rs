// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines optional JSON processing limits.

use crate::ResourceLimit;
use crate::ResourceQuantity;
use crate::StructureLimits;
use crate::json::JsonBudget;
use crate::json::JsonResource;

/// Optional limits for processing one JSON input or output.
///
/// `R` identifies the resource values reported in [`crate::BudgetError`], and
/// `Q` is the exact unsigned quantity used for all measurements. The default
/// configuration uses [`JsonResource`] and [`usize`]. Structural limits are
/// stored as a [`StructureLimits`] value and are reused directly by each
/// [`JsonBudget`] session.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JsonLimits<R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    /// Structural limits shared by the JSON budget.
    pub(crate) structure: StructureLimits<R, Q>,

    /// Optional inclusive maximum byte length of a complete input.
    pub(crate) max_input_bytes: Option<ResourceLimit<R, Q>>,

    /// Optional inclusive maximum byte length of a complete output.
    pub(crate) max_output_bytes: Option<ResourceLimit<R, Q>>,

    /// Optional inclusive maximum byte length of one string.
    pub(crate) max_string_bytes: Option<ResourceLimit<R, Q>>,

    /// Optional inclusive maximum byte length of one number representation.
    pub(crate) max_number_bytes: Option<ResourceLimit<R, Q>>,
}

impl<R, Q> Default for JsonLimits<R, Q>
where
    Q: ResourceQuantity,
{
    fn default() -> Self {
        Self {
            structure: StructureLimits {
                max_depth: None,
                max_nodes: None,
                max_sequence_items: None,
                max_map_entries: None,
                max_key_bytes: None,
            },
            max_input_bytes: None,
            max_output_bytes: None,
            max_string_bytes: None,
            max_number_bytes: None,
        }
    }
}

impl<R, Q> JsonLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an unconfigured custom-resource JSON limit set.
    #[inline]
    pub const fn empty() -> Self {
        Self {
            structure: StructureLimits::empty(),
            max_input_bytes: None,
            max_output_bytes: None,
            max_string_bytes: None,
            max_number_bytes: None,
        }
    }

    /// Configures the inclusive complete-input byte limit.
    #[inline]
    pub fn with_input_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.max_input_bytes = Some(limit);
        self
    }

    /// Configures the inclusive complete-output byte limit.
    #[inline]
    pub fn with_output_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.max_output_bytes = Some(limit);
        self
    }

    /// Configures the inclusive string byte limit.
    #[inline]
    pub fn with_string_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.max_string_bytes = Some(limit);
        self
    }

    /// Configures the inclusive number byte limit.
    #[inline]
    pub fn with_number_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.max_number_bytes = Some(limit);
        self
    }

    /// Replaces the structural limits used by this JSON configuration.
    #[inline]
    pub fn with_structure_limits<S>(mut self, limits: S) -> Self
    where
        S: Into<StructureLimits<R, Q>>,
    {
        self.structure = limits.into();
        self
    }

    /// Returns the structural limits represented by this JSON configuration.
    #[must_use = "the structural limits are part of the JSON budget configuration"]
    #[inline]
    pub fn structure_limits(&self) -> StructureLimits<R, Q>
    where
        R: Clone,
    {
        self.structure.clone()
    }

    /// Returns the configured maximum root-inclusive nesting depth.
    #[must_use]
    #[inline(always)]
    pub const fn max_depth(&self) -> Option<Q> {
        self.structure.max_depth()
    }

    /// Returns the configured maximum number of processed nodes.
    #[must_use]
    #[inline(always)]
    pub const fn max_nodes(&self) -> Option<Q> {
        self.structure.max_nodes()
    }

    /// Returns the configured maximum number of items in one array.
    #[must_use]
    #[inline(always)]
    pub const fn max_sequence_items(&self) -> Option<Q> {
        self.structure.max_sequence_items()
    }

    /// Returns the configured maximum number of entries in one object.
    #[must_use]
    #[inline(always)]
    pub const fn max_map_entries(&self) -> Option<Q> {
        self.structure.max_map_entries()
    }

    /// Returns the configured maximum byte length of one object key.
    #[must_use]
    #[inline(always)]
    pub const fn max_key_bytes(&self) -> Option<Q> {
        self.structure.max_key_bytes()
    }

    /// Returns the complete input-byte limit, when configured.
    #[must_use]
    #[inline(always)]
    pub const fn input_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_input_bytes.as_ref()
    }

    /// Returns the complete output-byte limit, when configured.
    #[must_use]
    #[inline(always)]
    pub const fn output_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_output_bytes.as_ref()
    }

    /// Returns the string-byte limit, when configured.
    #[must_use]
    #[inline(always)]
    pub const fn string_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_string_bytes.as_ref()
    }

    /// Returns the number-byte limit, when configured.
    #[must_use]
    #[inline(always)]
    pub const fn number_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_number_bytes.as_ref()
    }

    /// Returns the configured maximum complete-input byte length.
    #[must_use]
    #[inline(always)]
    pub fn max_input_bytes(&self) -> Option<Q> {
        limit_maximum(self.max_input_bytes.as_ref())
    }

    /// Returns the configured maximum complete-output byte length.
    #[must_use]
    #[inline(always)]
    pub fn max_output_bytes(&self) -> Option<Q> {
        limit_maximum(self.max_output_bytes.as_ref())
    }

    /// Returns the configured maximum string byte length.
    #[must_use]
    #[inline(always)]
    pub fn max_string_bytes(&self) -> Option<Q> {
        limit_maximum(self.max_string_bytes.as_ref())
    }

    /// Returns the configured maximum number byte length.
    #[must_use]
    #[inline(always)]
    pub fn max_number_bytes(&self) -> Option<Q> {
        limit_maximum(self.max_number_bytes.as_ref())
    }

    /// Creates an independent JSON budget session from these limits.
    #[inline]
    pub fn budget(&self) -> JsonBudget<R, Q>
    where
        R: Clone,
    {
        JsonBudget::new(self.clone())
    }
}

impl From<StructureLimits<crate::StructureResource, usize>>
    for StructureLimits<JsonResource, usize>
{
    fn from(limits: StructureLimits<crate::StructureResource, usize>) -> Self {
        let mut converted = Self::default();
        if let Some(maximum) = limits.max_depth() {
            converted =
                converted.with_depth_limit(ResourceLimit::new(JsonResource::Depth, maximum));
        }
        if let Some(maximum) = limits.max_nodes() {
            converted =
                converted.with_nodes_limit(ResourceLimit::new(JsonResource::Nodes, maximum));
        }
        if let Some(maximum) = limits.max_sequence_items() {
            converted = converted.with_sequence_items_limit(ResourceLimit::new(
                JsonResource::SequenceItems,
                maximum,
            ));
        }
        if let Some(maximum) = limits.max_map_entries() {
            converted = converted
                .with_map_entries_limit(ResourceLimit::new(JsonResource::MapEntries, maximum));
        }
        if let Some(maximum) = limits.max_key_bytes() {
            converted =
                converted.with_key_bytes_limit(ResourceLimit::new(JsonResource::KeyBytes, maximum));
        }
        converted
    }
}

impl JsonLimits<JsonResource, usize> {
    /// Creates a configuration with every JSON limit unconfigured.
    #[inline]
    pub const fn new() -> Self {
        Self {
            structure: StructureLimits {
                max_depth: None,
                max_nodes: None,
                max_sequence_items: None,
                max_map_entries: None,
                max_key_bytes: None,
            },
            max_input_bytes: None,
            max_output_bytes: None,
            max_string_bytes: None,
            max_number_bytes: None,
        }
    }

    /// Configures the inclusive complete-input byte limit.
    #[inline]
    pub const fn with_max_input_bytes(self, maximum: usize) -> Self {
        let mut limits = self;
        limits.max_input_bytes = Some(ResourceLimit::new(JsonResource::InputBytes, maximum));
        limits
    }

    /// Configures the inclusive complete-output byte limit.
    #[inline]
    pub const fn with_max_output_bytes(self, maximum: usize) -> Self {
        let mut limits = self;
        limits.max_output_bytes = Some(ResourceLimit::new(JsonResource::OutputBytes, maximum));
        limits
    }

    /// Configures the inclusive nesting-depth limit.
    #[inline]
    pub const fn with_max_depth(mut self, maximum: usize) -> Self {
        self.structure.max_depth = Some(ResourceLimit::new(JsonResource::Depth, maximum));
        self
    }

    /// Configures the cumulative node limit.
    #[inline]
    pub const fn with_max_nodes(mut self, maximum: usize) -> Self {
        self.structure.max_nodes = Some(ResourceLimit::new(JsonResource::Nodes, maximum));
        self
    }

    /// Configures the inclusive array-item limit.
    #[inline]
    pub const fn with_max_sequence_items(mut self, maximum: usize) -> Self {
        self.structure.max_sequence_items =
            Some(ResourceLimit::new(JsonResource::SequenceItems, maximum));
        self
    }

    /// Configures the inclusive object-entry limit.
    #[inline]
    pub const fn with_max_map_entries(mut self, maximum: usize) -> Self {
        self.structure.max_map_entries =
            Some(ResourceLimit::new(JsonResource::MapEntries, maximum));
        self
    }

    /// Configures the inclusive object-key byte limit.
    #[inline]
    pub const fn with_max_key_bytes(mut self, maximum: usize) -> Self {
        self.structure.max_key_bytes = Some(ResourceLimit::new(JsonResource::KeyBytes, maximum));
        self
    }

    /// Configures the inclusive string byte limit.
    #[inline]
    pub const fn with_max_string_bytes(self, maximum: usize) -> Self {
        let mut limits = self;
        limits.max_string_bytes = Some(ResourceLimit::new(JsonResource::StringBytes, maximum));
        limits
    }

    /// Configures the inclusive number byte limit.
    #[inline]
    pub const fn with_max_number_bytes(self, maximum: usize) -> Self {
        let mut limits = self;
        limits.max_number_bytes = Some(ResourceLimit::new(JsonResource::NumberBytes, maximum));
        limits
    }
}

/// Returns one optional resource maximum without exposing the resource value.
#[inline]
fn limit_maximum<R, Q>(limit: Option<&ResourceLimit<R, Q>>) -> Option<Q>
where
    Q: ResourceQuantity,
{
    limit.map(|limit| limit.maximum())
}
