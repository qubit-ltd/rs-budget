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
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Examples
///
/// ```
/// use qubit_budget::json::JsonMeasurement;
/// use qubit_budget::json::JsonValueLimits;
///
/// let mut budget = JsonValueLimits::builder()
///     .max_nodes(2_usize)
///     .max_string_bytes(8_usize)
///     .build()
///     .budget();
/// let mut transaction = budget.transaction();
/// transaction
///     .try_admit(JsonMeasurement::String { depth: 1, bytes: 5 })
///     .expect("the string should fit");
/// transaction.commit();
/// assert_eq!(budget.used_nodes(), Some(1));
/// ```
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
    ///
    /// # Returns
    ///
    /// Creates a limit set with every value dimension unconfigured.
    fn default() -> Self {
        Self::new()
    }
}

impl<R, Q> JsonValueLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an empty value limit set with no configured resource limits.
    ///
    /// # Returns
    ///
    /// Creates an empty value limit set with no configured resource limits.
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
    ///
    /// # Returns
    ///
    /// Creates a builder for JSON value limits.
    #[inline]
    #[must_use]
    pub const fn builder() -> JsonValueLimitsBuilder<R, Q> {
        JsonValueLimitsBuilder::new()
    }

    /// Converts these limits into a builder for further configuration.
    ///
    /// # Returns
    ///
    /// Converts these limits into a builder for further configuration.
    #[inline]
    #[must_use]
    pub const fn into_builder(self) -> JsonValueLimitsBuilder<R, Q> {
        JsonValueLimitsBuilder::from_limits(self)
    }

    /// Returns whether any JSON value or structural limit is configured.
    ///
    /// # Returns
    ///
    /// `true` when at least one value or nested structural dimension has a
    /// finite limit; otherwise `false`.
    #[must_use]
    #[inline(always)]
    pub const fn has_limits(&self) -> bool {
        self.structure.has_limits()
            || self.max_string_bytes.is_some()
            || self.max_number_bytes.is_some()
            || self.max_payload_bytes.is_some()
    }

    /// Borrows the structural limits used by this value configuration.
    ///
    /// # Returns
    ///
    /// Borrows the structural limits used by this value configuration.
    #[must_use]
    #[inline(always)]
    pub const fn structure_limits(&self) -> &StructureLimits<R, Q> {
        &self.structure
    }
    /// Consumes these value limits and returns their structural limits.
    ///
    /// # Returns
    ///
    /// Consumes these value limits and returns their structural limits.
    #[must_use]
    #[inline]
    pub fn into_structure_limits(self) -> StructureLimits<R, Q> {
        self.structure
    }
    /// Returns the configured root-inclusive nesting-depth maximum.
    ///
    /// # Returns
    ///
    /// Returns the configured root-inclusive nesting-depth maximum.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub const fn max_depth(&self) -> Option<Q> {
        self.structure.max_depth()
    }
    /// Returns the configured cumulative JSON-node maximum.
    ///
    /// # Returns
    ///
    /// Returns the configured cumulative JSON-node maximum.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub const fn max_nodes(&self) -> Option<Q> {
        self.structure.max_nodes()
    }
    /// Returns the configured maximum item count for one JSON array.
    ///
    /// # Returns
    ///
    /// Returns the configured maximum item count for one JSON array.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub const fn max_sequence_items(&self) -> Option<Q> {
        self.structure.max_sequence_items()
    }
    /// Returns the configured maximum entry count for one JSON object.
    ///
    /// # Returns
    ///
    /// Returns the configured maximum entry count for one JSON object.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub const fn max_map_entries(&self) -> Option<Q> {
        self.structure.max_map_entries()
    }
    /// Returns the configured maximum byte length for one JSON object key.
    ///
    /// # Returns
    ///
    /// Returns the configured maximum byte length for one JSON object key.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub const fn max_key_bytes(&self) -> Option<Q> {
        self.structure.max_key_bytes()
    }
    /// Returns the complete string-byte limit, when configured.
    ///
    /// # Returns
    ///
    /// Returns the complete string-byte limit, when configured.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub const fn string_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_string_bytes.as_ref()
    }
    /// Returns the complete number-byte limit, when configured.
    ///
    /// # Returns
    ///
    /// Returns the complete number-byte limit, when configured.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub const fn number_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_number_bytes.as_ref()
    }
    /// Returns the complete cumulative payload-byte limit, when configured.
    ///
    /// # Returns
    ///
    /// Returns the complete cumulative payload-byte limit, when configured.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub const fn payload_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_payload_bytes.as_ref()
    }
    /// Returns the configured maximum byte length for one string value.
    ///
    /// # Returns
    ///
    /// Returns the configured maximum byte length for one string value.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub const fn max_string_bytes(&self) -> Option<Q> {
        limit_maximum(self.max_string_bytes.as_ref())
    }
    /// Returns the configured maximum byte length for one number
    /// representation.
    ///
    /// # Returns
    ///
    /// Returns the configured maximum byte length for one number
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub const fn max_number_bytes(&self) -> Option<Q> {
        limit_maximum(self.max_number_bytes.as_ref())
    }
    /// Returns the configured cumulative payload-byte maximum.
    ///
    /// # Returns
    ///
    /// Returns the configured cumulative payload-byte maximum.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
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
    ///
    /// # Parameters
    ///
    /// * `measurement` - Native JSON measurement to convert or admit.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the operation completes successfully.
    ///
    /// # Errors
    ///
    /// Returns [`MeasuredBudgetError`] when a native measurement cannot fit `Q`
    /// or a configured limit rejects it.
    #[inline]
    pub fn check_point(&self, measurement: JsonMeasurement) -> Result<(), MeasuredBudgetError<R, Q>>
    where
        R: Clone,
    {
        PreparedJsonAdmission::prepare(self, measurement)?.check_point(self)
    }

    /// Creates a fresh mutable budget from these JSON value limits.
    ///
    /// # Returns
    ///
    /// Creates a fresh mutable budget from these JSON value limits.
    #[inline]
    #[must_use]
    pub fn budget(self) -> JsonValueBudget<R, Q> {
        JsonValueBudget::new(self)
    }

    /// Replaces the string-byte limit.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource-bound per-string byte limit to install.
    pub(super) fn set_string_bytes_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.max_string_bytes = Some(limit);
    }

    /// Replaces the number-byte limit.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource-bound per-number byte limit to install.
    pub(super) fn set_number_bytes_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.max_number_bytes = Some(limit);
    }

    /// Replaces the payload-byte limit.
    ///
    /// # Parameters
    ///
    /// * `limit` - Resource-bound cumulative payload-byte limit to install.
    pub(super) fn set_payload_bytes_limit(&mut self, limit: ResourceLimit<R, Q>) {
        self.max_payload_bytes = Some(limit);
    }

    /// Replaces the structural limits.
    ///
    /// # Parameters
    ///
    /// * `limits` - Structural limits to apply to JSON value processing.
    pub(super) fn set_structure_limits(&mut self, limits: StructureLimits<R, Q>) {
        self.structure = limits;
    }
}

/// Returns an optional limit maximum without exposing its resource identity.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Parameters
///
/// * `limit` - Optional resource-bound limit whose maximum is requested.
///
/// # Returns
///
/// Returns an optional limit maximum without exposing its resource identity.
///
/// `None` indicates that the supplied optional limit is unconfigured.
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
