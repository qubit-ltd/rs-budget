// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines JSON decoding limits.

use super::JsonResource;
use super::JsonValueLimits;
use crate::ResourceLimit;
use crate::ResourceQuantity;

/// Optional limits for one JSON decoding session.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JsonDecodeLimits<R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    input: Option<ResourceLimit<R, Q>>,
    normalized_input: Option<ResourceLimit<R, Q>>,
    value: JsonValueLimits<R, Q>,
}
impl<R, Q> Default for JsonDecodeLimits<R, Q>
where
    Q: ResourceQuantity,
{
    fn default() -> Self {
        Self::unconfigured()
    }
}
impl<R, Q> JsonDecodeLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an unconfigured generic decoding limit set.
    pub const fn unconfigured() -> Self {
        Self {
            input: None,
            normalized_input: None,
            value: JsonValueLimits::unconfigured(),
        }
    }
    /// Configures the cumulative raw input-byte budget.
    pub fn with_input_bytes_limit(
        mut self,
        limit: ResourceLimit<R, Q>,
    ) -> Self {
        self.input = Some(limit);
        self
    }
    /// Configures the cumulative normalized input-byte budget.
    pub fn with_normalized_input_bytes_limit(
        mut self,
        limit: ResourceLimit<R, Q>,
    ) -> Self {
        self.normalized_input = Some(limit);
        self
    }
    /// Replaces the direction-independent value limits for decoding.
    pub fn with_value_limits(mut self, limits: JsonValueLimits<R, Q>) -> Self {
        self.value = limits;
        self
    }
    /// Returns the complete raw input-byte limit, when configured.
    pub const fn input_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.input.as_ref()
    }
    /// Returns the complete normalized input-byte limit, when configured.
    pub const fn normalized_input_bytes_limit(
        &self,
    ) -> Option<&ResourceLimit<R, Q>> {
        self.normalized_input.as_ref()
    }
    /// Borrows the JSON value limits used for decoding.
    pub const fn value_limits(&self) -> &JsonValueLimits<R, Q> {
        &self.value
    }
    /// Consumes these decoding limits and returns their JSON value limits.
    pub fn into_value_limits(self) -> JsonValueLimits<R, Q> {
        self.value
    }
    /// Returns the configured raw input-byte maximum.
    pub const fn max_input_bytes(&self) -> Option<Q> {
        limit_maximum(self.input.as_ref())
    }
    /// Returns the configured normalized input-byte maximum.
    pub const fn max_normalized_input_bytes(&self) -> Option<Q> {
        limit_maximum(self.normalized_input.as_ref())
    }
}
impl JsonDecodeLimits<JsonResource, usize> {
    /// Creates an unconfigured JSON decoding limit set.
    pub const fn empty() -> Self {
        Self::unconfigured()
    }

    /// Configures the cumulative raw input-byte maximum.
    pub fn with_max_input_bytes(mut self, maximum: usize) -> Self {
        self.input =
            Some(ResourceLimit::new(JsonResource::InputBytes, maximum));
        self
    }

    /// Configures the cumulative normalized input-byte maximum.
    pub fn with_max_normalized_input_bytes(mut self, maximum: usize) -> Self {
        self.normalized_input = Some(ResourceLimit::new(
            JsonResource::NormalizedInputBytes,
            maximum,
        ));
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
/// Returns an optional maximum without exposing the resource identity.
const fn limit_maximum<R, Q>(limit: Option<&ResourceLimit<R, Q>>) -> Option<Q>
where
    Q: ResourceQuantity,
{
    match limit {
        Some(limit) => Some(limit.maximum()),
        None => None,
    }
}
