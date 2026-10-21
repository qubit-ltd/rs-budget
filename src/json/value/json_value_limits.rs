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
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut budget = JsonValueLimits::builder()
///     .max_nodes(2_usize)
///     .max_string_bytes(8_usize)
///     .build()
///     .budget();
/// let mut transaction = budget.transaction();
/// transaction
///     .try_admit(JsonMeasurement::String { depth: 1, bytes: 5 })
///     .expect("the string should fit");
/// transaction.commit()?;
/// assert_eq!(budget.used_nodes(), Some(1));
/// # Ok(()) }
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
    /// Creates a limit set with every structural and value-specific dimension
    /// unconfigured.
    ///
    /// # Returns
    ///
    /// A limit set with no configured resource limits.
    fn default() -> Self {
        Self::new()
    }
}

impl<R, Q> JsonValueLimits<R, Q>
where
    Q: ResourceQuantity,
{
    /// Creates an empty limit set with structural and value-specific limits
    /// unset.
    ///
    /// # Returns
    ///
    /// A limit set with no configured resource limits.
    #[inline]
    #[must_use = "the constructed limits should be used"]
    pub const fn new() -> Self {
        Self {
            structure: StructureLimits::new(),
            max_string_bytes: None,
            max_number_bytes: None,
            max_payload_bytes: None,
        }
    }
    /// Creates a builder with all JSON value limits unset.
    ///
    /// # Returns
    ///
    /// A builder ready to configure JSON value limits.
    #[inline]
    #[must_use]
    pub const fn builder() -> JsonValueLimitsBuilder<R, Q> {
        JsonValueLimitsBuilder::new()
    }

    /// Converts these limits into a builder while preserving their settings.
    ///
    /// # Returns
    ///
    /// A builder initialized with these limits.
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
    #[inline]
    pub const fn has_limits(&self) -> bool {
        self.structure.has_limits()
            || self.max_string_bytes.is_some()
            || self.max_number_bytes.is_some()
            || self.max_payload_bytes.is_some()
    }

    /// Borrows the structural limits that govern nesting, node counts,
    /// container sizes, and object-key sizes.
    ///
    /// # Returns
    ///
    /// The structural limits, borrowed for inspection.
    #[must_use]
    #[inline]
    pub const fn structure_limits(&self) -> &StructureLimits<R, Q> {
        &self.structure
    }
    /// Consumes these value limits and extracts the structural configuration.
    ///
    /// # Returns
    ///
    /// The structural limits, with value-specific byte limits discarded.
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
    /// `None` indicates that no maximum nesting depth is configured.
    #[inline]
    pub const fn max_depth(&self) -> Option<Q> {
        self.structure.max_depth()
    }
    /// Returns the configured cumulative JSON-node maximum.
    ///
    /// # Returns
    ///
    /// Returns the configured cumulative JSON-node maximum.
    ///
    /// `None` indicates that no maximum cumulative node count is configured.
    #[inline]
    pub const fn max_nodes(&self) -> Option<Q> {
        self.structure.max_nodes()
    }
    /// Returns the configured maximum item count for one JSON array.
    ///
    /// # Returns
    ///
    /// Returns the configured maximum item count for one JSON array.
    ///
    /// `None` indicates that no maximum array length is configured.
    #[inline]
    pub const fn max_sequence_items(&self) -> Option<Q> {
        self.structure.max_sequence_items()
    }
    /// Returns the configured maximum entry count for one JSON object.
    ///
    /// # Returns
    ///
    /// Returns the configured maximum entry count for one JSON object.
    ///
    /// `None` indicates that no maximum object entry count is configured.
    #[inline]
    pub const fn max_map_entries(&self) -> Option<Q> {
        self.structure.max_map_entries()
    }
    /// Returns the configured maximum byte length for one JSON object key.
    ///
    /// # Returns
    ///
    /// Returns the configured maximum byte length for one JSON object key.
    ///
    /// `None` indicates that no maximum object-key length is configured.
    #[inline]
    pub const fn max_key_bytes(&self) -> Option<Q> {
        self.structure.max_key_bytes()
    }
    /// Borrows the complete per-string byte limit, including its resource
    /// identity, when one is configured.
    ///
    /// # Returns
    ///
    /// The configured limit, or `None` when string-byte limiting is disabled.
    ///
    /// `None` indicates that string-byte limiting is disabled.
    #[inline]
    pub const fn string_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_string_bytes.as_ref()
    }
    /// Borrows the complete per-number byte limit, including its resource
    /// identity, when one is configured.
    ///
    /// # Returns
    ///
    /// The configured limit, or `None` when number-byte limiting is disabled.
    ///
    /// `None` indicates that number-byte limiting is disabled.
    #[inline]
    pub const fn number_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_number_bytes.as_ref()
    }
    /// Borrows the complete cumulative payload-byte limit, including its
    /// resource identity, when one is configured.
    ///
    /// # Returns
    ///
    /// The configured limit, or `None` when payload-byte limiting is disabled.
    ///
    /// `None` indicates that payload-byte limiting is disabled.
    #[inline]
    pub const fn payload_bytes_limit(&self) -> Option<&ResourceLimit<R, Q>> {
        self.max_payload_bytes.as_ref()
    }
    /// Returns the per-string byte maximum, without its resource identity.
    ///
    /// # Returns
    ///
    /// The configured maximum, or `None` when string-byte limiting is disabled.
    ///
    /// `None` indicates that string-byte limiting is disabled.
    #[inline]
    pub const fn max_string_bytes(&self) -> Option<Q> {
        limit_maximum(self.max_string_bytes.as_ref())
    }
    /// Returns the per-number representation byte maximum, without its
    /// resource identity.
    ///
    /// # Returns
    ///
    /// The configured maximum, or `None` when number-byte limiting is disabled.
    ///
    /// `None` indicates that number-byte limiting is disabled.
    #[inline]
    pub const fn max_number_bytes(&self) -> Option<Q> {
        limit_maximum(self.max_number_bytes.as_ref())
    }
    /// Returns the cumulative payload-byte maximum, without its resource
    /// identity.
    ///
    /// # Returns
    ///
    /// The configured maximum, or `None` when payload-byte limiting is
    /// disabled.
    ///
    /// `None` indicates that payload-byte limiting is disabled.
    #[inline]
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

    /// Consumes these limits to create a fresh mutable accounting budget.
    ///
    /// # Returns
    ///
    /// A budget initialized with these limits and zero usage.
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
#[inline]
const fn limit_maximum<R, Q>(limit: Option<&ResourceLimit<R, Q>>) -> Option<Q>
where
    Q: ResourceQuantity,
{
    match limit {
        Some(limit) => Some(limit.maximum()),
        None => None,
    }
}
