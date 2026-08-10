// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Enforces JSON processing limits during one processing session.

use crate::BudgetError;
use crate::ResourceQuantity;
use crate::StructureBudget;
use crate::json::JsonLimits;
use crate::json::JsonResource;
use crate::resource_limit::check_limit;

/// Mutable JSON accounting for one processing session.
///
/// The structural part is a [`StructureBudget`] stored directly, so all JSON
/// depth, node, container, and key checks preserve the same resource and
/// quantity types without an adapter or a second node counter.
#[must_use]
#[derive(Debug, PartialEq, Eq)]
pub struct JsonBudget<R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    /// Immutable JSON limits for this session.
    limits: JsonLimits<R, Q>,

    /// Shared structural accounting for this session.
    structure: StructureBudget<R, Q>,
}

impl<R, Q> JsonBudget<R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates a fresh budget session from one JSON limit configuration.
    #[inline]
    pub(crate) fn new(limits: JsonLimits<R, Q>) -> Self {
        let structure = limits.structure.budget();
        Self { limits, structure }
    }

    /// Checks the complete JSON input byte length.
    #[inline]
    pub fn check_input_bytes(
        &self,
        actual: Q,
    ) -> Result<(), BudgetError<R, Q>> {
        check_limit(self.limits.input_bytes_limit(), actual)
    }

    /// Checks the complete JSON output byte length.
    #[inline]
    pub fn check_output_bytes(
        &self,
        actual: Q,
    ) -> Result<(), BudgetError<R, Q>> {
        check_limit(self.limits.output_bytes_limit(), actual)
    }

    /// Checks root-inclusive JSON nesting depth.
    #[inline]
    pub fn check_depth(&self, actual: Q) -> Result<(), BudgetError<R, Q>> {
        self.structure.check_depth(actual)
    }

    /// Charges one processed JSON node.
    #[inline]
    pub fn charge_node(&mut self) -> Result<(), BudgetError<R, Q>> {
        self.structure.charge_node()
    }

    /// Charges several processed JSON nodes atomically.
    #[inline]
    pub fn charge_nodes(&mut self, amount: Q) -> Result<(), BudgetError<R, Q>> {
        self.structure.charge_nodes(amount)
    }

    /// Checks one JSON array item count.
    #[inline]
    pub fn check_sequence_items(
        &self,
        actual: Q,
    ) -> Result<(), BudgetError<R, Q>> {
        self.structure.check_sequence_items(actual)
    }

    /// Checks one JSON object entry count.
    #[inline]
    pub fn check_map_entries(
        &self,
        actual: Q,
    ) -> Result<(), BudgetError<R, Q>> {
        self.structure.check_map_entries(actual)
    }

    /// Checks one JSON object key byte length.
    #[inline]
    pub fn check_key_bytes(&self, actual: Q) -> Result<(), BudgetError<R, Q>> {
        self.structure.check_key_bytes(actual)
    }

    /// Checks one JSON string byte length.
    #[inline]
    pub fn check_string_bytes(
        &self,
        actual: Q,
    ) -> Result<(), BudgetError<R, Q>> {
        check_limit(self.limits.string_bytes_limit(), actual)
    }

    /// Checks one JSON number representation byte length.
    #[inline]
    pub fn check_number_bytes(
        &self,
        actual: Q,
    ) -> Result<(), BudgetError<R, Q>> {
        check_limit(self.limits.number_bytes_limit(), actual)
    }

    /// Checks a value depth and charges one node as one traversal step.
    #[inline]
    pub fn enter_node(&mut self, depth: Q) -> Result<(), BudgetError<R, Q>> {
        self.structure.enter_node(depth)
    }

    /// Checks an array size and charges one node atomically.
    #[inline]
    pub fn enter_array(
        &mut self,
        depth: Q,
        items: Q,
    ) -> Result<(), BudgetError<R, Q>> {
        self.structure.enter_sequence(depth, items)
    }

    /// Checks an object size and charges one node atomically.
    #[inline]
    pub fn enter_object(
        &mut self,
        depth: Q,
        entries: Q,
    ) -> Result<(), BudgetError<R, Q>> {
        self.structure.enter_map(depth, entries)
    }

    /// Returns the immutable limits copied into this session.
    #[must_use = "the configured limits determine which JSON charges can be accepted"]
    #[inline(always)]
    pub const fn limits(&self) -> &JsonLimits<R, Q> {
        &self.limits
    }

    /// Returns the shared structural budget.
    #[must_use = "the structural budget tracks shared JSON structure usage"]
    #[inline(always)]
    pub const fn structure_budget(&self) -> &StructureBudget<R, Q> {
        &self.structure
    }
}
