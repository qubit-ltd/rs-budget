// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines JSON encoding limits.

use super::JsonEncodeLimitsBuilder;
use crate::json::JsonResource;
use crate::json::JsonValueLimits;
use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;

/// Optional limits for one JSON encoding session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JsonEncodeLimits<R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    output: Option<ResourceLimit<R, Q>>,
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
    /// Creates an empty encoding limit set with no configured resource limits.
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

    /// Converts these limits into a builder for further configuration.
    #[inline]
    #[must_use]
    pub const fn into_builder(self) -> JsonEncodeLimitsBuilder<R, Q> {
        JsonEncodeLimitsBuilder::from_limits(self)
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

    /// Replaces the output-byte limit during builder composition.
    pub(super) fn set_output_bytes_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.output = Some(limit);
    }

    /// Replaces the JSON value limits during builder composition.
    pub(super) fn set_value_limits(&mut self, limits: JsonValueLimits<R, Q>) {
        self.value = limits;
    }
}
