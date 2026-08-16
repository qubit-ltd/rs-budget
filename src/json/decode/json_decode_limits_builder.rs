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
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { limits: JsonDecodeLimits::new() }
    }

    #[inline]
    #[must_use]
    pub fn input_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_input_bytes_limit(limit);
        self
    }

    #[inline]
    #[must_use]
    pub fn normalized_input_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_normalized_input_bytes_limit(limit);
        self
    }

    #[inline]
    #[must_use]
    pub fn value_limits(mut self, limits: JsonValueLimits<R, Q>) -> Self {
        self.limits.set_value_limits(limits);
        self
    }

    #[inline]
    #[must_use]
    pub fn build(self) -> JsonDecodeLimits<R, Q> {
        self.limits
    }
}

impl JsonDecodeLimitsBuilder<JsonResource, usize> {
    #[inline]
    #[must_use]
    pub fn max_input_bytes(mut self, maximum: usize) -> Self {
        self.limits.set_input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, maximum));
        self
    }

    #[inline]
    #[must_use]
    pub fn max_normalized_input_bytes(mut self, maximum: usize) -> Self {
        self.limits.set_normalized_input_bytes_limit(ResourceLimit::new(
            JsonResource::NormalizedInputBytes,
            maximum,
        ));
        self
    }

    #[inline]
    #[must_use]
    pub fn max_depth(self, maximum: usize) -> Self {
        self.map_value(|limits| limits.max_depth(maximum).build())
    }

    #[inline]
    #[must_use]
    pub fn max_nodes(self, maximum: usize) -> Self {
        self.map_value(|limits| limits.max_nodes(maximum).build())
    }

    #[inline]
    #[must_use]
    pub fn max_sequence_items(self, maximum: usize) -> Self {
        self.map_value(|limits| limits.max_sequence_items(maximum).build())
    }

    #[inline]
    #[must_use]
    pub fn max_map_entries(self, maximum: usize) -> Self {
        self.map_value(|limits| limits.max_map_entries(maximum).build())
    }

    #[inline]
    #[must_use]
    pub fn max_key_bytes(self, maximum: usize) -> Self {
        self.map_value(|limits| limits.max_key_bytes(maximum).build())
    }

    #[inline]
    #[must_use]
    pub fn max_string_bytes(self, maximum: usize) -> Self {
        self.map_value(|limits| limits.max_string_bytes(maximum).build())
    }

    #[inline]
    #[must_use]
    pub fn max_number_bytes(self, maximum: usize) -> Self {
        self.map_value(|limits| limits.max_number_bytes(maximum).build())
    }

    #[inline]
    #[must_use]
    pub fn max_payload_bytes(self, maximum: usize) -> Self {
        self.map_value(|limits| limits.max_payload_bytes(maximum).build())
    }

    fn map_value<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(crate::json::JsonValueLimitsBuilder) -> JsonValueLimits,
    {
        let value = *self.limits.value_limits();
        self.limits.set_value_limits(configure(value.into_builder()));
        self
    }
}
