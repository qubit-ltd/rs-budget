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
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Examples
///
/// ```
/// use qubit_budget::json::JsonDecodeLimits;
///
/// let limits = JsonDecodeLimits::builder().max_input_bytes(128_usize).max_depth(4_usize).build();
/// assert_eq!(limits.max_input_bytes(), Some(128));
/// assert_eq!(limits.value_limits().max_depth(), Some(4));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JsonDecodeLimits<R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    /// Optional maximum for bytes read from the original JSON input.
    input: Option<ResourceLimit<R, Q>>,
    /// Optional maximum for bytes retained after input normalization.
    normalized_input: Option<ResourceLimit<R, Q>>,
    /// Limits applied to the decoded JSON value and its structure.
    value: JsonValueLimits<R, Q>,
}

impl<R, Q> Default for JsonDecodeLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates decoding limits with every dimension unconfigured.
    ///
    /// # Returns
    ///
    /// Creates decoding limits with every dimension unconfigured.
    fn default() -> Self {
        Self::new()
    }
}

impl<R, Q> JsonDecodeLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an empty decoding limit set with no configured resource limits.
    ///
    /// # Returns
    ///
    /// Creates an empty decoding limit set with no configured resource limits.
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
    ///
    /// # Returns
    ///
    /// Creates a builder for JSON decoding limits.
    #[inline]
    #[must_use]
    pub const fn builder() -> JsonDecodeLimitsBuilder<R, Q> {
        JsonDecodeLimitsBuilder::new()
    }

    /// Converts these limits into a builder for further configuration.
    ///
    /// # Returns
    ///
    /// Converts these limits into a builder for further configuration.
    #[inline]
    #[must_use]
    pub const fn into_builder(self) -> JsonDecodeLimitsBuilder<R, Q> {
        JsonDecodeLimitsBuilder::from_limits(self)
    }

    /// Returns whether any decoding or nested value limit is configured.
    ///
    /// # Returns
    ///
    /// `true` when at least one input or nested value dimension has a finite
    /// limit; otherwise `false`.
    #[must_use]
    #[inline(always)]
    pub const fn has_limits(&self) -> bool {
        self.input.is_some() || self.normalized_input.is_some() || self.value.has_limits()
    }

    /// Returns the complete raw input-byte limit, when configured.
    ///
    /// # Returns
    ///
    /// Returns the complete raw input-byte limit, when configured.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub const fn input_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.input.as_ref()
    }

    /// Returns the complete normalized input-byte limit, when configured.
    ///
    /// # Returns
    ///
    /// Returns the complete normalized input-byte limit, when configured.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub const fn normalized_input_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.normalized_input.as_ref()
    }

    /// Borrows the JSON value limits used for decoding.
    ///
    /// # Returns
    ///
    /// Borrows the JSON value limits used for decoding.
    #[must_use]
    #[inline(always)]
    pub const fn value_limits(&self) -> &JsonValueLimits<R, Q> {
        &self.value
    }

    /// Consumes these decoding limits and returns their JSON value limits.
    ///
    /// # Returns
    ///
    /// Consumes these decoding limits and returns their JSON value limits.
    #[must_use]
    #[inline]
    pub fn into_value_limits(self) -> JsonValueLimits<R, Q> {
        self.value
    }

    /// Returns the configured raw input-byte maximum.
    ///
    /// # Returns
    ///
    /// Returns the configured raw input-byte maximum.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub const fn max_input_bytes(&self) -> Option<Q> {
        limit_maximum(self.input.as_ref())
    }

    /// Returns the configured normalized input-byte maximum.
    ///
    /// # Returns
    ///
    /// Returns the configured normalized input-byte maximum.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub const fn max_normalized_input_bytes(&self) -> Option<Q> {
        limit_maximum(self.normalized_input.as_ref())
    }

    /// Replaces the raw input-byte limit during builder composition.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource-bound raw input-byte limit to install.
    pub(super) fn set_input_bytes_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.input = Some(limit);
    }

    /// Replaces the normalized input-byte limit during builder composition.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource-bound normalized input-byte limit to install.
    pub(super) fn set_normalized_input_bytes_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.normalized_input = Some(limit);
    }

    /// Replaces the JSON value limits during builder composition.
    ///
    /// # Parameters
    ///
    /// * `limits` - JSON value limits to apply during decoding.
    pub(super) fn set_value_limits(&mut self, limits: JsonValueLimits<R, Q>) {
        self.value = limits;
    }
}

/// Extracts the maximum from an optional limit without exposing its resource.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Parameters
///
/// * `limit` - Optional resource-bound limit to inspect.
///
/// # Returns
///
/// `Some(maximum)` when the limit is configured, or `None` otherwise.
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
