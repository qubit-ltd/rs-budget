// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines JSON encoding limits.

use crate::ResourceLimit;
use crate::ResourceQuantity;

use super::JsonResource;
use super::JsonValueLimits;

/// Optional limits for one JSON encoding session.
#[must_use]
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
        Self::unconfigured()
    }
}
impl<R, Q> JsonEncodeLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an unconfigured generic encoding limit set.
    pub const fn unconfigured() -> Self {
        Self {
            output: None,
            value: JsonValueLimits::unconfigured(),
        }
    }
    /// Configures the cumulative output-byte budget.
    pub fn with_output_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.output = Some(limit);
        self
    }
    /// Replaces the direction-independent value limits for encoding.
    pub fn with_value_limits(mut self, limits: JsonValueLimits<R, Q>) -> Self {
        self.value = limits;
        self
    }
    /// Returns the complete output-byte limit, when configured.
    pub const fn output_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.output.as_ref()
    }
    /// Returns the JSON value limits used for encoding.
    pub fn value_limits(&self) -> JsonValueLimits<R, Q>
    where
        R: Clone,
    {
        self.value.clone()
    }
    /// Returns the configured output-byte maximum.
    pub const fn max_output_bytes(&self) -> Option<Q> {
        match self.output.as_ref() {
            Some(limit) => Some(limit.maximum()),
            None => None,
        }
    }
}
impl JsonEncodeLimits<JsonResource, usize> {
    /// Creates an unconfigured JSON encoding limit set.
    pub const fn empty() -> Self {
        Self::unconfigured()
    }
}
