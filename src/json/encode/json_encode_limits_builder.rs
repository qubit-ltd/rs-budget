// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Builds JSON encoding limits.

use super::JsonEncodeLimits;
use crate::json::JsonResource;
use crate::json::JsonValueLimits;
use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;

/// Builder for [`JsonEncodeLimits`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JsonEncodeLimitsBuilder<R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    limits: JsonEncodeLimits<R, Q>,
}

impl<R, Q> Default for JsonEncodeLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<R, Q> JsonEncodeLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an empty JSON encoding-limits builder.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            limits: JsonEncodeLimits::new(),
        }
    }

    /// Creates a builder retaining an existing limit configuration.
    #[inline]
    #[must_use]
    pub(crate) const fn from_limits(limits: JsonEncodeLimits<R, Q>) -> Self {
        Self { limits }
    }

    /// Sets the output-byte limit.
    #[inline]
    #[must_use]
    pub fn output_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_output_bytes_limit(limit);
        self
    }

    /// Sets the JSON value limits.
    #[inline]
    #[must_use]
    pub fn value_limits(mut self, limits: JsonValueLimits<R, Q>) -> Self {
        self.limits.set_value_limits(limits);
        self
    }

    /// Builds the configured JSON encoding limits.
    #[inline]
    #[must_use]
    pub fn build(self) -> JsonEncodeLimits<R, Q> {
        self.limits
    }
}

impl<Q> JsonEncodeLimitsBuilder<JsonResource, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an unconfigured encoding limit set using standard JSON types.
    #[must_use]
    pub const fn empty() -> Self {
        Self::new()
    }

    /// Sets the maximum output-byte count.
    #[inline]
    #[must_use]
    pub fn max_output_bytes(mut self, maximum: Q) -> Self {
        self.limits
            .set_output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, maximum));
        self
    }

    /// Sets the maximum nesting depth.
    #[inline]
    #[must_use]
    pub fn max_depth(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_depth(maximum).build())
    }

    /// Configures the cumulative maximum number of JSON nodes.
    #[inline]
    #[must_use]
    pub fn max_nodes(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_nodes(maximum).build())
    }

    /// Configures the maximum number of items in one JSON array.
    #[inline]
    #[must_use]
    pub fn max_sequence_items(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_sequence_items(maximum).build())
    }

    /// Configures the maximum number of entries in one JSON object.
    #[inline]
    #[must_use]
    pub fn max_map_entries(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_map_entries(maximum).build())
    }

    /// Configures the maximum UTF-8 byte length of one JSON object key.
    #[inline]
    #[must_use]
    pub fn max_key_bytes(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_key_bytes(maximum).build())
    }

    /// Configures the maximum UTF-8 byte length of one JSON string.
    #[inline]
    #[must_use]
    pub fn max_string_bytes(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_string_bytes(maximum).build())
    }

    /// Configures the maximum byte length of one JSON number representation.
    #[inline]
    #[must_use]
    pub fn max_number_bytes(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_number_bytes(maximum).build())
    }

    /// Configures the cumulative payload-byte maximum.
    #[inline]
    #[must_use]
    pub fn max_payload_bytes(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_payload_bytes(maximum).build())
    }

    fn map_value<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(
            crate::json::JsonValueLimitsBuilder<JsonResource, Q>,
        ) -> JsonValueLimits<JsonResource, Q>,
    {
        let value = *self.limits.value_limits();
        self.limits
            .set_value_limits(configure(value.into_builder()));
        self
    }
}
