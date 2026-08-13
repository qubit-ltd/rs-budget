// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines direction-independent limits for JSON values.

use super::JsonResource;
use super::JsonValueBudget;
use crate::ResourceLimit;
use crate::ResourceQuantity;
use crate::StructureLimits;

/// Optional limits for one JSON value traversal.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JsonValueLimits<R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    structure: StructureLimits<R, Q>,
    max_string_bytes: Option<ResourceLimit<R, Q>>,
    max_number_bytes: Option<ResourceLimit<R, Q>>,
    max_payload_bytes: Option<ResourceLimit<R, Q>>,
}

impl<R, Q> Default for JsonValueLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates a limit set with every value dimension unconfigured.
    fn default() -> Self {
        Self::unconfigured()
    }
}

impl<R, Q> JsonValueLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an unconfigured generic value limit set.
    pub const fn unconfigured() -> Self {
        Self {
            structure: StructureLimits::empty(),
            max_string_bytes: None,
            max_number_bytes: None,
            max_payload_bytes: None,
        }
    }
    /// Configures the inclusive byte limit for one string value.
    pub fn with_string_bytes_limit(
        mut self,
        limit: ResourceLimit<R, Q>,
    ) -> Self {
        self.max_string_bytes = Some(limit);
        self
    }
    /// Configures the inclusive byte limit for one number representation.
    pub fn with_number_bytes_limit(
        mut self,
        limit: ResourceLimit<R, Q>,
    ) -> Self {
        self.max_number_bytes = Some(limit);
        self
    }
    /// Configures the cumulative byte budget for keys, strings and numbers.
    pub fn with_payload_bytes_limit(
        mut self,
        limit: ResourceLimit<R, Q>,
    ) -> Self {
        self.max_payload_bytes = Some(limit);
        self
    }
    /// Replaces the structural limits used while processing JSON values.
    pub fn with_structure_limits<S>(mut self, limits: S) -> Self
    where
        S: Into<StructureLimits<R, Q>>,
    {
        self.structure = limits.into();
        self
    }
    /// Borrows the structural limits used by this value configuration.
    pub const fn structure_limits(&self) -> &StructureLimits<R, Q> {
        &self.structure
    }
    /// Consumes these value limits and returns their structural limits.
    pub fn into_structure_limits(self) -> StructureLimits<R, Q> {
        self.structure
    }
    /// Returns the configured root-inclusive nesting-depth maximum.
    pub const fn max_depth(&self) -> Option<Q> {
        self.structure.max_depth()
    }
    /// Returns the configured cumulative JSON-node maximum.
    pub const fn max_nodes(&self) -> Option<Q> {
        self.structure.max_nodes()
    }
    /// Returns the configured maximum item count for one JSON array.
    pub const fn max_sequence_items(&self) -> Option<Q> {
        self.structure.max_sequence_items()
    }
    /// Returns the configured maximum entry count for one JSON object.
    pub const fn max_map_entries(&self) -> Option<Q> {
        self.structure.max_map_entries()
    }
    /// Returns the configured maximum byte length for one JSON object key.
    pub const fn max_key_bytes(&self) -> Option<Q> {
        self.structure.max_key_bytes()
    }
    /// Returns the complete string-byte limit, when configured.
    pub const fn string_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_string_bytes.as_ref()
    }
    /// Returns the complete number-byte limit, when configured.
    pub const fn number_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_number_bytes.as_ref()
    }
    /// Returns the complete cumulative payload-byte limit, when configured.
    pub const fn payload_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_payload_bytes.as_ref()
    }
    /// Returns the configured maximum byte length for one string value.
    pub const fn max_string_bytes(&self) -> Option<Q> {
        limit_maximum(self.max_string_bytes.as_ref())
    }
    /// Returns the configured maximum byte length for one number
    /// representation.
    pub const fn max_number_bytes(&self) -> Option<Q> {
        limit_maximum(self.max_number_bytes.as_ref())
    }
    /// Returns the configured cumulative payload-byte maximum.
    pub const fn max_payload_bytes(&self) -> Option<Q> {
        limit_maximum(self.max_payload_bytes.as_ref())
    }
}

impl JsonValueLimits<JsonResource, usize> {
    /// Creates a value limit set with every JSON value dimension unconfigured.
    pub const fn empty() -> Self {
        Self::unconfigured()
    }

    /// Configures the inclusive maximum nesting depth.
    pub fn with_max_depth(mut self, maximum: usize) -> Self {
        self.structure = self
            .structure
            .with_depth_limit(ResourceLimit::new(JsonResource::Depth, maximum));
        self
    }

    /// Configures the cumulative maximum number of JSON nodes.
    pub fn with_max_nodes(mut self, maximum: usize) -> Self {
        self.structure = self
            .structure
            .with_nodes_limit(ResourceLimit::new(JsonResource::Nodes, maximum));
        self
    }

    /// Configures the maximum number of items in one JSON array.
    pub fn with_max_sequence_items(mut self, maximum: usize) -> Self {
        self.structure = self.structure.with_sequence_items_limit(
            ResourceLimit::new(JsonResource::SequenceItems, maximum),
        );
        self
    }

    /// Configures the maximum number of entries in one JSON object.
    pub fn with_max_map_entries(mut self, maximum: usize) -> Self {
        self.structure = self.structure.with_map_entries_limit(
            ResourceLimit::new(JsonResource::MapEntries, maximum),
        );
        self
    }

    /// Configures the maximum UTF-8 byte length of one JSON object key.
    pub fn with_max_key_bytes(mut self, maximum: usize) -> Self {
        self.structure = self.structure.with_key_bytes_limit(
            ResourceLimit::new(JsonResource::KeyBytes, maximum),
        );
        self
    }

    /// Configures the maximum UTF-8 byte length of one JSON string.
    pub fn with_max_string_bytes(mut self, maximum: usize) -> Self {
        self.max_string_bytes =
            Some(ResourceLimit::new(JsonResource::StringBytes, maximum));
        self
    }

    /// Configures the maximum byte length of one JSON number representation.
    pub fn with_max_number_bytes(mut self, maximum: usize) -> Self {
        self.max_number_bytes =
            Some(ResourceLimit::new(JsonResource::NumberBytes, maximum));
        self
    }

    /// Configures the cumulative payload-byte maximum.
    pub fn with_max_payload_bytes(mut self, maximum: usize) -> Self {
        self.max_payload_bytes =
            Some(ResourceLimit::new(JsonResource::PayloadBytes, maximum));
        self
    }

    /// Creates a fresh mutable budget from these JSON value limits.
    pub fn budget(self) -> JsonValueBudget {
        JsonValueBudget::new(self)
    }
}

/// Returns an optional limit maximum without exposing its resource identity.
const fn limit_maximum<R, Q>(limit: Option<&ResourceLimit<R, Q>>) -> Option<Q>
where
    Q: ResourceQuantity,
{
    match limit {
        Some(limit) => Some(limit.maximum()),
        None => None,
    }
}
