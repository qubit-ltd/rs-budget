// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines direction-independent limits for JSON values.

use super::JsonValueBudget;
use super::JsonValueLimitsBuilder;
use super::internal::PreparedJsonAdmission;
use crate::json::JsonMeasurement;
use crate::json::JsonResource;
use crate::resource::MeasuredBudgetError;
use crate::resource::ResourceLimit;
use crate::resource::ResourceQuantity;
use crate::structure::StructureLimits;

/// Optional limits for one JSON value traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JsonValueLimits<R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    /// Structural limits for depth, nodes, containers, and keys.
    structure: StructureLimits<R, Q>,
    /// Optional per-string byte limit.
    max_string_bytes: Option<ResourceLimit<R, Q>>,
    /// Optional per-number byte limit.
    max_number_bytes: Option<ResourceLimit<R, Q>>,
    /// Optional cumulative payload byte limit.
    max_payload_bytes: Option<ResourceLimit<R, Q>>,
}

impl<R, Q> Default for JsonValueLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates a limit set with every value dimension unconfigured.
    fn default() -> Self {
        Self::new()
    }
}

impl<R, Q> JsonValueLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an unconfigured generic value limit set.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            structure: StructureLimits::new(),
            max_string_bytes: None,
            max_number_bytes: None,
            max_payload_bytes: None,
        }
    }
    /// Creates a builder for JSON value limits.
    #[inline]
    #[must_use]
    pub const fn builder() -> JsonValueLimitsBuilder<R, Q> {
        JsonValueLimitsBuilder::new()
    }

    /// Converts these limits into a builder for further configuration.
    #[inline]
    #[must_use]
    pub(crate) const fn into_builder(self) -> JsonValueLimitsBuilder<R, Q> {
        JsonValueLimitsBuilder::from_limits(self)
    }
    /// Borrows the structural limits used by this value configuration.
    #[must_use]
    #[inline(always)]
    pub const fn structure_limits(&self) -> &StructureLimits<R, Q> {
        &self.structure
    }
    /// Consumes these value limits and returns their structural limits.
    #[must_use]
    #[inline]
    pub fn into_structure_limits(self) -> StructureLimits<R, Q> {
        self.structure
    }
    /// Returns the configured root-inclusive nesting-depth maximum.
    #[must_use]
    #[inline(always)]
    pub const fn max_depth(&self) -> Option<Q> {
        self.structure.max_depth()
    }
    /// Returns the configured cumulative JSON-node maximum.
    #[must_use]
    #[inline(always)]
    pub const fn max_nodes(&self) -> Option<Q> {
        self.structure.max_nodes()
    }
    /// Returns the configured maximum item count for one JSON array.
    #[must_use]
    #[inline(always)]
    pub const fn max_sequence_items(&self) -> Option<Q> {
        self.structure.max_sequence_items()
    }
    /// Returns the configured maximum entry count for one JSON object.
    #[must_use]
    #[inline(always)]
    pub const fn max_map_entries(&self) -> Option<Q> {
        self.structure.max_map_entries()
    }
    /// Returns the configured maximum byte length for one JSON object key.
    #[must_use]
    #[inline(always)]
    pub const fn max_key_bytes(&self) -> Option<Q> {
        self.structure.max_key_bytes()
    }
    /// Returns the complete string-byte limit, when configured.
    #[must_use]
    #[inline(always)]
    pub const fn string_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_string_bytes.as_ref()
    }
    /// Returns the complete number-byte limit, when configured.
    #[must_use]
    #[inline(always)]
    pub const fn number_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_number_bytes.as_ref()
    }
    /// Returns the complete cumulative payload-byte limit, when configured.
    #[must_use]
    #[inline(always)]
    pub const fn payload_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_payload_bytes.as_ref()
    }
    /// Returns the configured maximum byte length for one string value.
    #[must_use]
    #[inline(always)]
    pub const fn max_string_bytes(&self) -> Option<Q> {
        limit_maximum(self.max_string_bytes.as_ref())
    }
    /// Returns the configured maximum byte length for one number
    /// representation.
    #[must_use]
    #[inline(always)]
    pub const fn max_number_bytes(&self) -> Option<Q> {
        limit_maximum(self.max_number_bytes.as_ref())
    }
    /// Returns the configured cumulative payload-byte maximum.
    #[must_use]
    #[inline(always)]
    pub const fn max_payload_bytes(&self) -> Option<Q> {
        limit_maximum(self.max_payload_bytes.as_ref())
    }

    /// Validates one native JSON measurement against point limits only.
    ///
    /// The measurement is converted only for configured dimensions, then
    /// checked in conversion, depth, and variant-specific point-limit order.
    /// Cumulative limits such as `max_nodes` and `max_payload_bytes` are not
    /// charged or checked by this method.
    ///
    /// Returns conversion or point-limit errors retaining their associated
    /// resource identity.
    #[inline]
    pub fn check_point(
        &self,
        measurement: JsonMeasurement,
    ) -> Result<(), MeasuredBudgetError<R, Q>>
    where
        R: Clone,
    {
        PreparedJsonAdmission::prepare(self, measurement)?.check_point(self)
    }

    /// Creates a fresh mutable budget from these JSON value limits.
    #[inline]
    #[must_use]
    pub fn budget(self) -> JsonValueBudget<R, Q> {
        JsonValueBudget::new(self)
    }

    /// Replaces the string-byte limit.
    pub(super) fn set_string_bytes_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.max_string_bytes = Some(limit);
    }

    /// Replaces the number-byte limit.
    pub(super) fn set_number_bytes_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.max_number_bytes = Some(limit);
    }

    /// Replaces the payload-byte limit.
    pub(super) fn set_payload_bytes_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.max_payload_bytes = Some(limit);
    }

    /// Replaces the structural limits.
    pub(super) fn set_structure_limits(&mut self, limits: StructureLimits<R, Q>) {
        self.structure = limits;
    }
}

impl JsonValueLimits<JsonResource, usize> {
    /// Creates an unconfigured value limit set using the standard JSON
    /// resource types and `usize` measurements.
    #[must_use]
    pub const fn empty() -> Self {
        Self::new()
    }
}

/// Returns an optional limit maximum without exposing its resource identity.
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
