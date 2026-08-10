// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Enforces JSON processing limits during one processing session.

use crate::BudgetError;
use crate::ResourceBudget;
use crate::ResourceLimit;
use crate::json::JsonLimits;
use crate::json::JsonResource;

/// Mutable JSON accounting for one processing session.
///
/// Point limits do not accumulate between calls. Node charges consume the
/// session's finite node budget when one was configured.
#[must_use]
#[derive(Debug, PartialEq, Eq)]
pub struct JsonBudget {
    /// Immutable point limits for this session.
    limits: JsonLimits,

    /// Optional cumulative node budget for this session.
    nodes: Option<ResourceBudget<JsonResource, usize>>,
}

impl JsonBudget {
    /// Creates a fresh budget session from one JSON limit configuration.
    ///
    /// # Parameters
    ///
    /// * `limits` - Immutable configuration copied into the new session.
    ///
    /// # Returns
    ///
    /// A session whose optional node budget has its full configured capacity.
    #[inline]
    pub(crate) fn new(limits: JsonLimits) -> Self {
        Self {
            nodes: limits.max_nodes.map(ResourceBudget::from_limit),
            limits,
        }
    }

    /// Checks the complete JSON input byte length against its maximum.
    ///
    /// # Parameters
    ///
    /// * `actual` - Byte length of the complete JSON input.
    ///
    /// # Returns
    ///
    /// `Ok(())` when input bytes are unconfigured or fit the inclusive
    /// maximum.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::LimitExceeded`] with
    /// [`JsonResource::InputBytes`] when `actual` exceeds the configured
    /// maximum. This method does not mutate the session.
    #[inline]
    pub fn check_input_bytes(
        &self,
        actual: usize,
    ) -> Result<(), BudgetError<JsonResource, usize>> {
        check_limit(self.limits.max_input_bytes, actual)
    }

    /// Checks root-inclusive JSON nesting depth against its maximum.
    ///
    /// # Parameters
    ///
    /// * `actual` - Root-inclusive depth of the current JSON value.
    ///
    /// # Returns
    ///
    /// `Ok(())` when depth is unconfigured or fits the inclusive maximum.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::LimitExceeded`] with [`JsonResource::Depth`]
    /// when `actual` exceeds the configured maximum. This method does not
    /// mutate the session.
    #[inline]
    pub fn check_depth(
        &self,
        actual: usize,
    ) -> Result<(), BudgetError<JsonResource, usize>> {
        check_limit(self.limits.max_depth, actual)
    }

    /// Charges one processed JSON node to this session's cumulative budget.
    ///
    /// # Returns
    ///
    /// `Ok(())` when nodes are unconfigured or the next node fits.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::Insufficient`] with [`JsonResource::Nodes`]
    /// when no node capacity remains. A failed charge leaves the node budget
    /// unchanged.
    #[inline]
    pub fn charge_node(
        &mut self,
    ) -> Result<(), BudgetError<JsonResource, usize>> {
        match &mut self.nodes {
            Some(nodes) => nodes.try_consume(1),
            None => Ok(()),
        }
    }

    /// Checks one JSON array item count against its maximum.
    ///
    /// # Parameters
    ///
    /// * `actual` - Item count of the current JSON array.
    ///
    /// # Returns
    ///
    /// `Ok(())` when array items are unconfigured or fit the inclusive
    /// maximum.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::LimitExceeded`] with
    /// [`JsonResource::SequenceItems`] when `actual` exceeds the configured
    /// maximum. This method does not mutate the session.
    #[inline]
    pub fn check_sequence_items(
        &self,
        actual: usize,
    ) -> Result<(), BudgetError<JsonResource, usize>> {
        check_limit(self.limits.max_sequence_items, actual)
    }

    /// Checks one JSON object entry count against its maximum.
    ///
    /// # Parameters
    ///
    /// * `actual` - Entry count of the current JSON object.
    ///
    /// # Returns
    ///
    /// `Ok(())` when object entries are unconfigured or fit the inclusive
    /// maximum.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::LimitExceeded`] with
    /// [`JsonResource::MapEntries`] when `actual` exceeds the configured
    /// maximum. This method does not mutate the session.
    #[inline]
    pub fn check_map_entries(
        &self,
        actual: usize,
    ) -> Result<(), BudgetError<JsonResource, usize>> {
        check_limit(self.limits.max_map_entries, actual)
    }

    /// Checks one JSON string byte length against its maximum.
    ///
    /// # Parameters
    ///
    /// * `actual` - Byte length of the current JSON string.
    ///
    /// # Returns
    ///
    /// `Ok(())` when string bytes are unconfigured or fit the inclusive
    /// maximum.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::LimitExceeded`] with
    /// [`JsonResource::StringBytes`] when `actual` exceeds the configured
    /// maximum. This method does not mutate the session.
    #[inline]
    pub fn check_string_bytes(
        &self,
        actual: usize,
    ) -> Result<(), BudgetError<JsonResource, usize>> {
        check_limit(self.limits.max_string_bytes, actual)
    }

    /// Checks one JSON number representation byte length against its maximum.
    ///
    /// # Parameters
    ///
    /// * `actual` - Byte length of the current JSON number representation.
    ///
    /// # Returns
    ///
    /// `Ok(())` when number bytes are unconfigured or fit the inclusive
    /// maximum.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetError::LimitExceeded`] with
    /// [`JsonResource::NumberBytes`] when `actual` exceeds the configured
    /// maximum. This method does not mutate the session.
    #[inline]
    pub fn check_number_bytes(
        &self,
        actual: usize,
    ) -> Result<(), BudgetError<JsonResource, usize>> {
        check_limit(self.limits.max_number_bytes, actual)
    }
}

/// Checks an optional JSON point limit.
///
/// # Parameters
///
/// * `limit` - Optional inclusive resource limit for one JSON measurement.
/// * `actual` - Observed measurement to validate.
///
/// # Returns
///
/// `Ok(())` when `limit` is absent or accepts `actual`.
///
/// # Errors
///
/// Returns [`BudgetError::LimitExceeded`] when `limit` is configured and
/// `actual` exceeds its inclusive maximum. This helper has no side effects.
#[inline]
fn check_limit(
    limit: Option<ResourceLimit<JsonResource, usize>>,
    actual: usize,
) -> Result<(), BudgetError<JsonResource, usize>> {
    match limit {
        Some(limit) => limit.check(actual),
        None => Ok(()),
    }
}
