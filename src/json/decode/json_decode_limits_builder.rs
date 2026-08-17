// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Builds JSON decoding limits.

use super::JsonDecodeLimits;
use crate::json::JsonResource;
use crate::json::JsonValueLimits;
use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;

/// Builder for [`JsonDecodeLimits`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JsonDecodeLimitsBuilder<R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    limits: JsonDecodeLimits<R, Q>,
}

impl<R, Q> Default for JsonDecodeLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<R, Q> JsonDecodeLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an empty JSON decoding-limits builder.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            limits: JsonDecodeLimits::new(),
        }
    }

    /// Creates a builder retaining an existing limit configuration.
    #[inline]
    #[must_use]
    pub(crate) const fn from_limits(limits: JsonDecodeLimits<R, Q>) -> Self {
        Self { limits }
    }

    /// Sets the raw input-byte limit.
    #[inline]
    #[must_use]
    pub fn input_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_input_bytes_limit(limit);
        self
    }

    /// Sets the normalized input-byte limit.
    #[inline]
    #[must_use]
    pub fn normalized_input_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_normalized_input_bytes_limit(limit);
        self
    }

    /// Sets the JSON value limits.
    #[inline]
    #[must_use]
    pub fn value_limits(mut self, limits: JsonValueLimits<R, Q>) -> Self {
        self.limits.set_value_limits(limits);
        self
    }

    /// Builds the configured JSON decoding limits.
    #[inline]
    #[must_use]
    pub fn build(self) -> JsonDecodeLimits<R, Q> {
        self.limits
    }
}

impl<Q> JsonDecodeLimitsBuilder<JsonResource, Q>
where
    Q: ResourceQuantity,
{
    /// Sets the maximum raw input-byte count.
    #[inline]
    #[must_use]
    pub fn max_input_bytes(mut self, maximum: Q) -> Self {
        self.limits
            .set_input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, maximum));
        self
    }

    /// Sets the maximum normalized input-byte count.
    #[inline]
    #[must_use]
    pub fn max_normalized_input_bytes(mut self, maximum: Q) -> Self {
        self.limits
            .set_normalized_input_bytes_limit(ResourceLimit::new(
                JsonResource::NormalizedInputBytes,
                maximum,
            ));
        self
    }

    /// Sets the maximum nesting depth.
    #[inline]
    #[must_use]
    pub fn max_depth(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_depth(maximum).build())
    }

    /// Sets the maximum number of JSON nodes.
    #[inline]
    #[must_use]
    pub fn max_nodes(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_nodes(maximum).build())
    }

    /// Sets the maximum number of items in one JSON array.
    #[inline]
    #[must_use]
    pub fn max_sequence_items(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_sequence_items(maximum).build())
    }

    /// Sets the maximum number of entries in one JSON object.
    #[inline]
    #[must_use]
    pub fn max_map_entries(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_map_entries(maximum).build())
    }

    /// Sets the maximum UTF-8 byte length of one JSON object key.
    #[inline]
    #[must_use]
    pub fn max_key_bytes(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_key_bytes(maximum).build())
    }

    /// Sets the maximum UTF-8 byte length of one JSON string.
    #[inline]
    #[must_use]
    pub fn max_string_bytes(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_string_bytes(maximum).build())
    }

    /// Sets the maximum byte length of one JSON number representation.
    #[inline]
    #[must_use]
    pub fn max_number_bytes(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_number_bytes(maximum).build())
    }

    /// Sets the cumulative payload-byte maximum.
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
