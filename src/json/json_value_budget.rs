// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tracks mutable accounting for one JSON value traversal.

use crate::BudgetError;
use crate::MeasuredBudgetError;
use crate::ResourceBudget;
use crate::ResourceLimit;
use crate::ResourceQuantity;
use crate::StructureBudget;

use super::JsonResource;
use super::JsonValueLimits;

/// Mutable accounting for JSON structure and value payloads.
#[must_use]
#[derive(Debug, PartialEq, Eq)]
pub struct JsonValueBudget<R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    limits: JsonValueLimits<R, Q>,
    structure: StructureBudget<R, Q>,
    payload: Option<ResourceBudget<R, Q>>,
}

impl<R, Q> JsonValueBudget<R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates a fresh JSON value budget from immutable limits.
    pub fn new(limits: JsonValueLimits<R, Q>) -> Self {
        let structure = limits.structure_limits().budget();
        let payload = limits
            .payload_bytes_limit()
            .cloned()
            .map(ResourceBudget::from_limit);
        Self {
            limits,
            structure,
            payload,
        }
    }

    /// Restores the budget to the state configured by its original limits.
    pub fn reset(&mut self) {
        *self = Self::new(self.limits.clone());
    }

    /// Checks and charges one scalar node.
    pub fn enter_node(&mut self, depth: Q) -> Result<(), BudgetError<R, Q>> {
        self.structure.enter_node(depth)
    }

    /// Converts and admits one JSON scalar measured with native depth.
    pub fn enter_node_usize(&mut self, depth: usize) -> Result<(), MeasuredBudgetError<R, Q>> {
        let depth = self.convert_usize(depth, self.limits.structure_limits().depth_limit())?;
        self.enter_node(depth).map_err(MeasuredBudgetError::from)
    }

    /// Checks and charges one array node.
    pub fn enter_array(&mut self, depth: Q, items: Q) -> Result<(), BudgetError<R, Q>> {
        self.structure.enter_sequence(depth, items)
    }

    /// Converts and admits one JSON array measured with native quantities.
    pub fn enter_array_usize(
        &mut self,
        depth: usize,
        items: usize,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        let limits = self.limits.structure_limits();
        let depth = self.convert_usize(depth, limits.depth_limit())?;
        let items = self.convert_usize(items, limits.sequence_items_limit())?;
        self.enter_array(depth, items)
            .map_err(MeasuredBudgetError::from)
    }

    /// Checks and charges one object node.
    pub fn enter_object(&mut self, depth: Q, entries: Q) -> Result<(), BudgetError<R, Q>> {
        self.structure.enter_map(depth, entries)
    }

    /// Converts and admits one JSON object measured with native quantities.
    pub fn enter_object_usize(
        &mut self,
        depth: usize,
        entries: usize,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        let limits = self.limits.structure_limits();
        let depth = self.convert_usize(depth, limits.depth_limit())?;
        let entries = self.convert_usize(entries, limits.map_entries_limit())?;
        self.enter_object(depth, entries)
            .map_err(MeasuredBudgetError::from)
    }

    /// Checks and atomically charges one string node and its payload.
    pub fn enter_string(&mut self, depth: Q, bytes: Q) -> Result<(), BudgetError<R, Q>> {
        self.structure.check_depth(depth)?;
        self.structure.check_node_available()?;
        self.check_string_bytes(bytes)?;
        self.check_payload_bytes(bytes)?;
        self.structure.charge_node()?;
        self.consume_payload_bytes(bytes)
    }

    /// Checks and atomically charges one number node and its payload.
    pub fn enter_number(&mut self, depth: Q, bytes: Q) -> Result<(), BudgetError<R, Q>> {
        self.structure.check_depth(depth)?;
        self.structure.check_node_available()?;
        self.check_number_bytes(bytes)?;
        self.check_payload_bytes(bytes)?;
        self.structure.charge_node()?;
        self.consume_payload_bytes(bytes)
    }

    /// Converts and charges one string node measured with native quantities.
    pub fn enter_string_usize(
        &mut self,
        depth: usize,
        bytes: usize,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        let depth = self.convert_usize(depth, self.limits.structure_limits().depth_limit())?;
        let bytes = self.convert_payload_usize(bytes, self.limits.string_bytes_limit())?;
        self.enter_string(depth, bytes)
            .map_err(MeasuredBudgetError::from)
    }

    /// Converts and charges one number node measured with native quantities.
    pub fn enter_number_usize(
        &mut self,
        depth: usize,
        bytes: usize,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        let depth = self.convert_usize(depth, self.limits.structure_limits().depth_limit())?;
        let bytes = self.convert_payload_usize(bytes, self.limits.number_bytes_limit())?;
        self.enter_number(depth, bytes)
            .map_err(MeasuredBudgetError::from)
    }

    /// Returns the structural accounting state.
    pub const fn structure_budget(&self) -> &StructureBudget<R, Q> {
        &self.structure
    }

    /// Returns the cumulative payload budget when configured.
    pub const fn payload_budget(&self) -> Option<&ResourceBudget<R, Q>> {
        self.payload.as_ref()
    }

    /// Returns the immutable limits used by this traversal.
    pub const fn limits(&self) -> &JsonValueLimits<R, Q> {
        &self.limits
    }

    /// Checks one JSON tree depth without charging any resource.
    pub fn check_depth(&self, actual: Q) -> Result<(), BudgetError<R, Q>> {
        self.structure.check_depth(actual)
    }

    /// Charges one JSON node after the caller has performed any required checks.
    pub fn charge_node(&mut self) -> Result<(), BudgetError<R, Q>> {
        self.structure.charge_node()
    }

    /// Checks the item count of one JSON array.
    pub fn check_sequence_items(&self, actual: Q) -> Result<(), BudgetError<R, Q>> {
        self.structure.check_sequence_items(actual)
    }

    /// Checks the entry count of one JSON object.
    pub fn check_map_entries(&self, actual: Q) -> Result<(), BudgetError<R, Q>> {
        self.structure.check_map_entries(actual)
    }

    /// Checks the byte length of one JSON object key.
    pub fn check_key_bytes(&self, actual: Q) -> Result<(), BudgetError<R, Q>> {
        self.structure.check_key_bytes(actual)
    }

    /// Checks and consumes one object key's payload bytes.
    pub fn consume_key_bytes(&mut self, amount: Q) -> Result<(), BudgetError<R, Q>> {
        self.check_key_bytes(amount)?;
        self.consume_payload_bytes(amount)
    }

    /// Checks and consumes one string value's payload bytes.
    pub fn consume_string_bytes(&mut self, amount: Q) -> Result<(), BudgetError<R, Q>> {
        self.check_string_bytes(amount)?;
        self.consume_payload_bytes(amount)
    }

    /// Checks and consumes one number representation's payload bytes.
    pub fn consume_number_bytes(&mut self, amount: Q) -> Result<(), BudgetError<R, Q>> {
        self.check_number_bytes(amount)?;
        self.consume_payload_bytes(amount)
    }

    /// Converts and checks one native array-item count.
    pub fn check_sequence_items_usize(
        &self,
        actual: usize,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        let actual = self.convert_usize(
            actual,
            self.limits.structure_limits().sequence_items_limit(),
        )?;
        self.check_sequence_items(actual)
            .map_err(MeasuredBudgetError::from)
    }

    /// Converts and checks one native object-entry count.
    pub fn check_map_entries_usize(&self, actual: usize) -> Result<(), MeasuredBudgetError<R, Q>> {
        let actual =
            self.convert_usize(actual, self.limits.structure_limits().map_entries_limit())?;
        self.check_map_entries(actual)
            .map_err(MeasuredBudgetError::from)
    }

    /// Converts and consumes native object-key bytes.
    pub fn consume_key_bytes_usize(
        &mut self,
        amount: usize,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        let amount =
            self.convert_payload_usize(amount, self.limits.structure_limits().key_bytes_limit())?;
        self.consume_key_bytes(amount)
            .map_err(MeasuredBudgetError::from)
    }

    /// Converts and consumes native string bytes.
    pub fn consume_string_bytes_usize(
        &mut self,
        amount: usize,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        let amount = self.convert_payload_usize(amount, self.limits.string_bytes_limit())?;
        self.consume_string_bytes(amount)
            .map_err(MeasuredBudgetError::from)
    }

    /// Converts and consumes native number bytes.
    pub fn consume_number_bytes_usize(
        &mut self,
        amount: usize,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        let amount = self.convert_payload_usize(amount, self.limits.number_bytes_limit())?;
        self.consume_number_bytes(amount)
            .map_err(MeasuredBudgetError::from)
    }

    /// Converts and checks native object-key bytes without consuming payload.
    pub fn check_key_bytes_usize(&self, amount: usize) -> Result<(), MeasuredBudgetError<R, Q>> {
        let amount =
            self.convert_usize(amount, self.limits.structure_limits().key_bytes_limit())?;
        self.check_key_bytes(amount)
            .map_err(MeasuredBudgetError::from)
    }

    /// Converts and checks native string bytes without consuming payload.
    pub fn check_string_bytes_usize(&self, amount: usize) -> Result<(), MeasuredBudgetError<R, Q>> {
        let amount = self.convert_usize(amount, self.limits.string_bytes_limit())?;
        self.check_string_bytes(amount)
            .map_err(MeasuredBudgetError::from)
    }

    /// Converts and checks native number bytes without consuming payload.
    pub fn check_number_bytes_usize(&self, amount: usize) -> Result<(), MeasuredBudgetError<R, Q>> {
        let amount = self.convert_usize(amount, self.limits.number_bytes_limit())?;
        self.check_number_bytes(amount)
            .map_err(MeasuredBudgetError::from)
    }

    /// Checks the byte length of one string value.
    pub fn check_string_bytes(&self, bytes: Q) -> Result<(), BudgetError<R, Q>> {
        self.limits
            .string_bytes_limit()
            .map_or(Ok(()), |limit| limit.check(bytes))
    }

    /// Checks the byte length of one number representation.
    pub fn check_number_bytes(&self, bytes: Q) -> Result<(), BudgetError<R, Q>> {
        self.limits
            .number_bytes_limit()
            .map_or(Ok(()), |limit| limit.check(bytes))
    }

    /// Checks the cumulative payload budget without changing it.
    fn check_payload_bytes(&self, bytes: Q) -> Result<(), BudgetError<R, Q>> {
        self.payload
            .as_ref()
            .map_or(Ok(()), |payload| payload.check_available(bytes))
    }

    /// Consumes a payload amount known to have already passed every check.
    fn consume_payload_bytes(&mut self, bytes: Q) -> Result<(), BudgetError<R, Q>> {
        self.payload
            .as_mut()
            .map_or(Ok(()), |payload| payload.try_consume(bytes))
    }

    /// Converts a native quantity only when its associated limit is configured.
    fn convert_usize(
        &self,
        amount: usize,
        limit: Option<&ResourceLimit<R, Q>>,
    ) -> Result<Q, MeasuredBudgetError<R, Q>> {
        let Some(limit) = limit else {
            return Ok(Q::ZERO);
        };
        Q::try_from_usize(amount)
            .map_err(|source| MeasuredBudgetError::quantity(limit.resource().clone(), source))
    }

    /// Converts a payload quantity when either its point or cumulative limit is configured.
    fn convert_payload_usize(
        &self,
        amount: usize,
        point_limit: Option<&ResourceLimit<R, Q>>,
    ) -> Result<Q, MeasuredBudgetError<R, Q>> {
        if let Some(limit) = point_limit {
            return Q::try_from_usize(amount)
                .map_err(|source| MeasuredBudgetError::quantity(limit.resource().clone(), source));
        }
        if let Some(payload) = &self.payload {
            return Q::try_from_usize(amount).map_err(|source| {
                MeasuredBudgetError::quantity(payload.resource().clone(), source)
            });
        }
        Ok(Q::ZERO)
    }
}
