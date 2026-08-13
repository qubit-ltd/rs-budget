// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines JSON decoding limits.

use crate::ResourceLimit;
use crate::ResourceQuantity;

use super::JsonResource;
use super::JsonValueLimits;

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
    pub fn with_input_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.input = Some(limit);
        self
    }
    /// Configures the cumulative normalized input-byte budget.
    pub fn with_normalized_input_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
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
    pub const fn normalized_input_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.normalized_input.as_ref()
    }
    /// Returns the JSON value limits used for decoding.
    pub fn value_limits(&self) -> JsonValueLimits<R, Q>
    where
        R: Clone,
    {
        self.value.clone()
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
