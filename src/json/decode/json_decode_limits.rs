// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines JSON decoding limits.

use super::JsonDecodeLimitsBuilder;
use crate::json::JsonResource;
use crate::json::JsonValueLimits;
use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;

/// Optional limits for one JSON decoding session.
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
        Self::new()
    }
}

impl<R, Q> JsonDecodeLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an unconfigured generic decoding limit set.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            input: None,
            normalized_input: None,
            value: JsonValueLimits::new(),
        }
    }

    /// Creates a builder for JSON decoding limits.
    #[inline]
    #[must_use]
    pub const fn builder() -> JsonDecodeLimitsBuilder<R, Q> {
        JsonDecodeLimitsBuilder::new()
    }

    /// Converts these limits into a builder for further configuration.
    #[inline]
    #[must_use]
    pub const fn into_builder(self) -> JsonDecodeLimitsBuilder<R, Q> {
        JsonDecodeLimitsBuilder::from_limits(self)
    }

    /// Returns the complete raw input-byte limit, when configured.
    #[must_use]
    #[inline(always)]
    pub const fn input_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.input.as_ref()
    }

    /// Returns the complete normalized input-byte limit, when configured.
    #[must_use]
    #[inline(always)]
    pub const fn normalized_input_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.normalized_input.as_ref()
    }

    /// Borrows the JSON value limits used for decoding.
    #[must_use]
    #[inline(always)]
    pub const fn value_limits(&self) -> &JsonValueLimits<R, Q> {
        &self.value
    }

    /// Consumes these decoding limits and returns their JSON value limits.
    #[must_use]
    #[inline]
    pub fn into_value_limits(self) -> JsonValueLimits<R, Q> {
        self.value
    }

    /// Returns the configured raw input-byte maximum.
    #[must_use]
    #[inline(always)]
    pub const fn max_input_bytes(&self) -> Option<Q> {
        limit_maximum(self.input.as_ref())
    }

    /// Returns the configured normalized input-byte maximum.
    #[must_use]
    #[inline(always)]
    pub const fn max_normalized_input_bytes(&self) -> Option<Q> {
        limit_maximum(self.normalized_input.as_ref())
    }

    /// Replaces the raw input-byte limit during builder composition.
    pub(super) fn set_input_bytes_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.input = Some(limit);
    }

    /// Replaces the normalized input-byte limit during builder composition.
    pub(super) fn set_normalized_input_bytes_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.normalized_input = Some(limit);
    }

    /// Replaces the JSON value limits during builder composition.
    pub(super) fn set_value_limits(&mut self, limits: JsonValueLimits<R, Q>) {
        self.value = limits;
    }
}

impl JsonDecodeLimits<JsonResource, usize> {
    /// Creates an unconfigured decoding limit set using standard JSON types.
    #[must_use]
    pub const fn empty() -> Self {
        Self::new()
    }
}

#[inline(always)]
const fn limit_maximum<R, Q>(limit: Option<&ResourceLimit<R, Q>>) -> Option<Q>
where
    Q: ResourceQuantity,
{
    match limit {
        Some(limit) => Some(limit.maximum()),
        None => None,
    }
}
