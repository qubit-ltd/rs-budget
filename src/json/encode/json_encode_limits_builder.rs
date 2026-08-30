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
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Examples
///
/// ```
/// use qubit_budget::json::JsonEncodeLimitsBuilder;
///
/// let limits = JsonEncodeLimitsBuilder::new().max_output_bytes(128_usize).build();
/// assert_eq!(limits.max_output_bytes(), Some(128));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JsonEncodeLimitsBuilder<R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    /// Limit configuration accumulated by chained builder calls.
    limits: JsonEncodeLimits<R, Q>,
}

impl<R, Q> Default for JsonEncodeLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an empty builder through the standard [`Default`] interface.
    ///
    /// # Returns
    ///
    /// Creates an empty builder through the standard [`Default`] interface.
    fn default() -> Self {
        Self::new()
    }
}

impl<R, Q> JsonEncodeLimitsBuilder<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an empty JSON encoding-limits builder.
    ///
    /// # Returns
    ///
    /// Creates an empty JSON encoding-limits builder.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            limits: JsonEncodeLimits::new(),
        }
    }

    /// Creates a builder retaining an existing limit configuration.
    ///
    /// # Parameters
    ///
    /// * `limits` - Existing encoding limits whose configuration is copied into
    ///   this builder.
    ///
    /// # Returns
    ///
    /// Creates a builder retaining an existing limit configuration.
    #[inline]
    #[must_use]
    pub(crate) const fn from_limits(limits: JsonEncodeLimits<R, Q>) -> Self {
        Self { limits }
    }

    /// Sets the output-byte limit.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource-bound output-byte limit to install.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn output_bytes_limit(mut self, limit: ResourceLimit<R, Q>) -> Self {
        self.limits.set_output_bytes_limit(limit);
        self
    }

    /// Sets the JSON value limits.
    ///
    /// # Parameters
    ///
    /// * `limits` - JSON value limits to apply during encoding.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn value_limits(mut self, limits: JsonValueLimits<R, Q>) -> Self {
        self.limits.set_value_limits(limits);
        self
    }

    /// Builds the configured JSON encoding limits.
    ///
    /// # Returns
    ///
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
    /// Sets the maximum output-byte count.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Inclusive maximum to configure.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn max_output_bytes(mut self, maximum: Q) -> Self {
        self.limits
            .set_output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, maximum));
        self
    }

    /// Sets the maximum nesting depth.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Inclusive maximum to configure.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn max_depth(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_depth(maximum).build())
    }

    /// Configures the cumulative maximum number of JSON nodes.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Inclusive maximum to configure.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn max_nodes(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_nodes(maximum).build())
    }

    /// Configures the maximum number of items in one JSON array.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Inclusive maximum to configure.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn max_sequence_items(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_sequence_items(maximum).build())
    }

    /// Configures the maximum number of entries in one JSON object.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Inclusive maximum to configure.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn max_map_entries(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_map_entries(maximum).build())
    }

    /// Configures the maximum UTF-8 byte length of one JSON object key.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Inclusive maximum to configure.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn max_key_bytes(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_key_bytes(maximum).build())
    }

    /// Configures the maximum UTF-8 byte length of one JSON string.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Inclusive maximum to configure.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn max_string_bytes(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_string_bytes(maximum).build())
    }

    /// Configures the maximum byte length of one JSON number representation.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Inclusive maximum to configure.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn max_number_bytes(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_number_bytes(maximum).build())
    }

    /// Configures the cumulative payload-byte maximum.
    ///
    /// # Parameters
    ///
    /// * `maximum` - Inclusive maximum to configure.
    ///
    /// # Returns
    ///
    /// The builder with the described setting applied.
    #[inline]
    #[must_use]
    pub fn max_payload_bytes(self, maximum: Q) -> Self {
        self.map_value(|limits| limits.max_payload_bytes(maximum).build())
    }

    /// Applies one transformation to the nested JSON value limits.
    ///
    /// # Type Parameters
    ///
    /// * `F` - Closure used to configure the nested value-limit builder.
    ///
    /// # Parameters
    ///
    /// * `configure` - Transformation applied to the current value-limit
    ///   builder.
    ///
    /// # Returns
    ///
    /// This encoding builder with the transformed value limits installed.
    fn map_value<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(crate::json::JsonValueLimitsBuilder<JsonResource, Q>) -> JsonValueLimits<JsonResource, Q>,
    {
        let value = *self.limits.value_limits();
        self.limits.set_value_limits(configure(value.into_builder()));
        self
    }
}
