// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow multiple-public-types
//! Defines JSON encoding limits.

use super::JsonResource;
use super::JsonValueLimits;
use super::JsonValueLimitsBuilder;
use crate::ResourceLimit;
use crate::ResourceQuantity;

/// Optional limits for one JSON encoding session.
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
    /// Creates an encoding limit set with every dimension unconfigured.
    fn default() -> Self {
        Self::new()
    }
}
impl<R, Q> JsonEncodeLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an unconfigured generic encoding limit set.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            output: None,
            value: JsonValueLimits::new(),
        }
    }
    /// Creates a builder for JSON encoding limits.
    #[inline]
    #[must_use]
    pub const fn builder() -> JsonEncodeLimitsBuilder<R, Q> {
        JsonEncodeLimitsBuilder::new()
    }
    /// Returns the complete output-byte limit, when configured.
    #[must_use]
    #[inline(always)]
    pub const fn output_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.output.as_ref()
    }
    /// Borrows the JSON value limits used for encoding.
    #[must_use]
    #[inline(always)]
    pub const fn value_limits(&self) -> &JsonValueLimits<R, Q> {
        &self.value
    }
    /// Consumes these encoding limits and returns their JSON value limits.
    #[must_use]
    #[inline]
    pub fn into_value_limits(self) -> JsonValueLimits<R, Q> {
        self.value
    }
    /// Returns the configured output-byte maximum.
    #[must_use]
    #[inline(always)]
    pub const fn max_output_bytes(&self) -> Option<Q> {
        match self.output.as_ref() {
            Some(limit) => Some(limit.maximum()),
            None => None,
        }
    }
}
/// Builder for [`JsonEncodeLimits`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JsonEncodeLimitsBuilder<R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    pub(crate) limits: JsonEncodeLimits<R, Q>,
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

    /// Sets the output-byte limit.
    #[inline]
    #[must_use]
    pub fn output_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.output = Some(limit);
        self
    }

    /// Sets the JSON value limits.
    #[inline]
    #[must_use]
    pub fn value_limits(mut self, limits: JsonValueLimits<R, Q>) -> Self {
        self.limits.value = limits;
        self
    }

    /// Builds the configured JSON encoding limits.
    #[inline]
    #[must_use]
    pub fn build(self) -> JsonEncodeLimits<R, Q> {
        self.limits
    }
}

impl JsonEncodeLimits<JsonResource, usize> {
    /// Creates an unconfigured encoding limit set using standard JSON types.
    #[must_use]
    pub const fn empty() -> Self {
        Self::new()
    }
}

impl JsonEncodeLimitsBuilder<JsonResource, usize> {
    /// Creates an unconfigured encoding limit set using the standard JSON
    /// resource types and `usize` measurements.
    #[must_use]
    pub const fn empty() -> Self {
        Self::new()
    }
    /// Sets the maximum output-byte count.
    #[inline]
    #[must_use]
    pub fn max_output_bytes(mut self, maximum: usize) -> Self {
        self.limits.output =
            Some(ResourceLimit::new(JsonResource::OutputBytes, maximum));
        self
    }

    /// Sets the maximum nesting depth.
    #[inline]
    #[must_use]
    pub fn max_depth(mut self, maximum: usize) -> Self {
        self.limits.value = JsonValueLimitsBuilder {
            limits: self.limits.value,
        }
        .max_depth(maximum)
        .build();
        self
    }

    /// Configures the cumulative maximum number of JSON nodes.
    #[inline]
    #[must_use]
    pub fn max_nodes(mut self, maximum: usize) -> Self {
        self.limits.value = JsonValueLimitsBuilder {
            limits: self.limits.value,
        }
        .max_nodes(maximum)
        .build();
        self
    }

    /// Configures the maximum number of items in one JSON array.
    #[inline]
    #[must_use]
    pub fn max_sequence_items(mut self, maximum: usize) -> Self {
        self.limits.value = JsonValueLimitsBuilder {
            limits: self.limits.value,
        }
        .max_sequence_items(maximum)
        .build();
        self
    }

    /// Configures the maximum number of entries in one JSON object.
    #[inline]
    #[must_use]
    pub fn max_map_entries(mut self, maximum: usize) -> Self {
        self.limits.value = JsonValueLimitsBuilder {
            limits: self.limits.value,
        }
        .max_map_entries(maximum)
        .build();
        self
    }

    /// Configures the maximum UTF-8 byte length of one JSON object key.
    #[inline]
    #[must_use]
    pub fn max_key_bytes(mut self, maximum: usize) -> Self {
        self.limits.value = JsonValueLimitsBuilder {
            limits: self.limits.value,
        }
        .max_key_bytes(maximum)
        .build();
        self
    }

    /// Configures the maximum UTF-8 byte length of one JSON string.
    #[inline]
    #[must_use]
    pub fn max_string_bytes(mut self, maximum: usize) -> Self {
        self.limits.value = JsonValueLimitsBuilder {
            limits: self.limits.value,
        }
        .max_string_bytes(maximum)
        .build();
        self
    }

    /// Configures the maximum byte length of one JSON number representation.
    #[inline]
    #[must_use]
    pub fn max_number_bytes(mut self, maximum: usize) -> Self {
        self.limits.value = JsonValueLimitsBuilder {
            limits: self.limits.value,
        }
        .max_number_bytes(maximum)
        .build();
        self
    }

    /// Configures the cumulative payload-byte maximum.
    #[inline]
    #[must_use]
    pub fn max_payload_bytes(mut self, maximum: usize) -> Self {
        self.limits.value = JsonValueLimitsBuilder {
            limits: self.limits.value,
        }
        .max_payload_bytes(maximum)
        .build();
        self
    }
}
