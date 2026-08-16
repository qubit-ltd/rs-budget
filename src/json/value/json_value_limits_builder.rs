// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Builds direction-independent limits for JSON values.

use super::JsonValueBudget;
use super::JsonValueLimits;
use crate::json::JsonResource;
use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;
use crate::structure::StructureLimits;

/// Builder for [`JsonValueLimits`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JsonValueLimitsBuilder<R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    limits: JsonValueLimits<R, Q>,
}

impl<R, Q> Default for JsonValueLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<R, Q> JsonValueLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an empty JSON value-limits builder.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            limits: JsonValueLimits::new(),
        }
    }

    /// Restores a builder from an existing limit set.
    #[inline]
    #[must_use]
    pub(crate) const fn from_limits(limits: JsonValueLimits<R, Q>) -> Self {
        Self { limits }
    }

    /// Sets the string-byte limit.
    #[inline]
    #[must_use]
    pub fn string_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_string_bytes_limit(limit);
        self
    }

    /// Sets the number-byte limit.
    #[inline]
    #[must_use]
    pub fn number_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_number_bytes_limit(limit);
        self
    }

    /// Sets the payload-byte limit.
    #[inline]
    #[must_use]
    pub fn payload_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_payload_bytes_limit(limit);
        self
    }

    /// Sets the structural limits.
    #[inline]
    #[must_use]
    pub fn structure_limits<S>(mut self, limits: S) -> Self
    where
        S: Into<StructureLimits<R, Q>>,
    {
        self.limits.set_structure_limits(limits.into());
        self
    }

    /// Builds the configured JSON value limits.
    #[inline]
    #[must_use]
    pub fn build(self) -> JsonValueLimits<R, Q> {
        self.limits
    }

    /// Creates a mutable JSON value budget from this builder.
    #[inline]
    #[must_use]
    pub fn budget(self) -> JsonValueBudget<R, Q> {
        self.build().budget()
    }
}

impl<Q> JsonValueLimitsBuilder<JsonResource, Q>
where
    Q: ResourceQuantity,
{
    /// Sets the maximum nesting depth.
    #[inline]
    #[must_use]
    pub fn max_depth(mut self, maximum: Q) -> Self {
        let structure = self
            .limits
            .structure_limits()
            .into_builder()
            .depth_limit(ResourceLimit::new(JsonResource::Depth, maximum))
            .build();
        self.limits.set_structure_limits(structure);
        self
    }

    /// Sets the maximum number of JSON nodes.
    #[inline]
    #[must_use]
    pub fn max_nodes(mut self, maximum: Q) -> Self {
        let structure = self
            .limits
            .structure_limits()
            .into_builder()
            .nodes_limit(ResourceLimit::new(JsonResource::Nodes, maximum))
            .build();
        self.limits.set_structure_limits(structure);
        self
    }

    /// Sets the maximum number of items in one JSON array.
    #[inline]
    #[must_use]
    pub fn max_sequence_items(mut self, maximum: Q) -> Self {
        let structure = self
            .limits
            .structure_limits()
            .into_builder()
            .sequence_items_limit(ResourceLimit::new(JsonResource::SequenceItems, maximum))
            .build();
        self.limits.set_structure_limits(structure);
        self
    }

    /// Sets the maximum number of entries in one JSON object.
    #[inline]
    #[must_use]
    pub fn max_map_entries(mut self, maximum: Q) -> Self {
        let structure = self
            .limits
            .structure_limits()
            .into_builder()
            .map_entries_limit(ResourceLimit::new(JsonResource::MapEntries, maximum))
            .build();
        self.limits.set_structure_limits(structure);
        self
    }

    /// Sets the maximum UTF-8 byte length of one JSON object key.
    #[inline]
    #[must_use]
    pub fn max_key_bytes(mut self, maximum: Q) -> Self {
        let structure = self
            .limits
            .structure_limits()
            .into_builder()
            .key_bytes_limit(ResourceLimit::new(JsonResource::KeyBytes, maximum))
            .build();
        self.limits.set_structure_limits(structure);
        self
    }

    /// Sets the maximum UTF-8 byte length of one JSON string.
    #[inline]
    #[must_use]
    pub fn max_string_bytes(mut self, maximum: Q) -> Self {
        self.limits
            .set_string_bytes_limit(ResourceLimit::new(JsonResource::StringBytes, maximum));
        self
    }

    /// Sets the maximum byte length of one JSON number representation.
    #[inline]
    #[must_use]
    pub fn max_number_bytes(mut self, maximum: Q) -> Self {
        self.limits
            .set_number_bytes_limit(ResourceLimit::new(JsonResource::NumberBytes, maximum));
        self
    }

    /// Sets the cumulative payload-byte maximum.
    #[inline]
    #[must_use]
    pub fn max_payload_bytes(mut self, maximum: Q) -> Self {
        self.limits
            .set_payload_bytes_limit(ResourceLimit::new(JsonResource::PayloadBytes, maximum));
        self
    }
}
