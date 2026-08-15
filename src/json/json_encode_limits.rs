// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines JSON encoding limits.

use super::JsonResource;
use super::JsonValueLimits;
use crate::ResourceLimit;
use crate::ResourceQuantity;

/// Optional limits for one JSON encoding session.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JsonEncodeLimits<R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    /// Optional cumulative output-byte limit.
    output: Option<ResourceLimit<R, Q>>,
    /// Direction-independent JSON value limits.
    value: JsonValueLimits<R, Q>,
}
impl<R, Q> Default for JsonEncodeLimits<R, Q>
where
    Q: ResourceQuantity,
{
    fn default() -> Self {
        Self::new()
    }
}
impl<R, Q> JsonEncodeLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an unconfigured generic encoding limit set.
    pub const fn new() -> Self {
        Self {
            output: None,
            value: JsonValueLimits::new(),
        }
    }
    pub const fn unconfigured() -> Self {
        Self::new()
    }
    /// Configures the cumulative output-byte budget.
    pub fn with_output_bytes_limit(
        mut self,
        limit: ResourceLimit<R, Q>,
    ) -> Self {
        self.output = Some(limit);
        self
    }
    /// Replaces the direction-independent value limits for encoding.
    pub fn with_value_limits(mut self, limits: JsonValueLimits<R, Q>) -> Self {
        self.value = limits;
        self
    }
    /// Returns the complete output-byte limit, when configured.
    #[must_use]
    pub const fn output_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.output.as_ref()
    }
    /// Borrows the JSON value limits used for encoding.
    #[must_use = "the value limits determine encode traversal constraints"]
    pub const fn value_limits(&self) -> &JsonValueLimits<R, Q> {
        &self.value
    }
    /// Consumes these encoding limits and returns their JSON value limits.
    #[must_use = "the returned value limits can configure a JSON value budget"]
    pub fn into_value_limits(self) -> JsonValueLimits<R, Q> {
        self.value
    }
    /// Returns the configured output-byte maximum.
    #[must_use]
    pub const fn max_output_bytes(&self) -> Option<Q> {
        match self.output.as_ref() {
            Some(limit) => Some(limit.maximum()),
            None => None,
        }
    }
}
impl JsonEncodeLimits<JsonResource, usize> {
    pub const fn empty() -> Self {
        Self::new()
    }
    /// Configures the cumulative output-byte maximum.
    pub fn with_max_output_bytes(mut self, maximum: usize) -> Self {
        self.output =
            Some(ResourceLimit::new(JsonResource::OutputBytes, maximum));
        self
    }

    /// Configures the inclusive maximum nesting depth.
    pub fn with_max_depth(mut self, maximum: usize) -> Self {
        self.value = self.value.with_max_depth(maximum);
        self
    }

    /// Configures the cumulative maximum number of JSON nodes.
    pub fn with_max_nodes(mut self, maximum: usize) -> Self {
        self.value = self.value.with_max_nodes(maximum);
        self
    }

    /// Configures the maximum number of items in one JSON array.
    pub fn with_max_sequence_items(mut self, maximum: usize) -> Self {
        self.value = self.value.with_max_sequence_items(maximum);
        self
    }

    /// Configures the maximum number of entries in one JSON object.
    pub fn with_max_map_entries(mut self, maximum: usize) -> Self {
        self.value = self.value.with_max_map_entries(maximum);
        self
    }

    /// Configures the maximum UTF-8 byte length of one JSON object key.
    pub fn with_max_key_bytes(mut self, maximum: usize) -> Self {
        self.value = self.value.with_max_key_bytes(maximum);
        self
    }

    /// Configures the maximum UTF-8 byte length of one JSON string.
    pub fn with_max_string_bytes(mut self, maximum: usize) -> Self {
        self.value = self.value.with_max_string_bytes(maximum);
        self
    }

    /// Configures the maximum byte length of one JSON number representation.
    pub fn with_max_number_bytes(mut self, maximum: usize) -> Self {
        self.value = self.value.with_max_number_bytes(maximum);
        self
    }

    /// Configures the cumulative payload-byte maximum.
    pub fn with_max_payload_bytes(mut self, maximum: usize) -> Self {
        self.value = self.value.with_max_payload_bytes(maximum);
        self
    }
}
