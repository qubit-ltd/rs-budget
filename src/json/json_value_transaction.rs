// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Transactional admission for one complete JSON value.

use super::JsonContainerKind;
use super::JsonMeasurement;
use super::JsonValueBudget;
use super::internal::JsonValueState;
use super::internal::PreparedJsonAdmission;
use crate::InsufficientBudgetError;
use crate::MeasuredBudgetError;
use crate::ResourceQuantity;

/// A transaction that stages JSON-value accounting until explicitly committed.
///
/// Dropping this value, including during unwinding, discards its fixed-size
/// working state and leaves the target budget's committed state unchanged.
#[must_use]
pub struct JsonValueTransaction<'a, R, Q>
where
    Q: ResourceQuantity,
{
    /// Budget that receives the staged state if [`Self::commit`] is called.
    target: &'a mut JsonValueBudget<R, Q>,
    /// Fixed-size state changed by successful staged admissions.
    working: JsonValueState<Q>,
}

impl<'a, R, Q> JsonValueTransaction<'a, R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates a transaction using a snapshot of `target`'s committed state.
    pub(super) const fn new(target: &'a mut JsonValueBudget<R, Q>) -> Self {
        Self {
            working: target.state,
            target,
        }
    }

    /// Stages one native JSON measurement when all applicable limits allow it.
    ///
    /// Point limits are checked before cumulative node and payload capacity.
    /// Within cumulative accounting, node capacity is checked before payload.
    ///
    /// Returns conversion, point-limit, or cumulative-budget errors with their
    /// configured resource identity. Any error leaves this transaction's
    /// working state unchanged and does not affect the committed budget.
    pub fn try_admit(
        &mut self,
        measurement: JsonMeasurement,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        let prepared =
            PreparedJsonAdmission::prepare(self.target.limits(), measurement)?;
        prepared.check_point(self.target.limits())?;
        self.check_cumulative(prepared)?;
        self.apply(prepared);
        Ok(())
    }

    /// Checks one prospective JSON container count without mutating this
    /// transaction.
    ///
    /// # Parameters
    ///
    /// * `kind` - Container dimension selected for the point-limit check.
    /// * `prospective` - Count that would result if the next child were
    ///   entered.
    ///
    /// # Errors
    ///
    /// Returns a quantity conversion error when `prospective` cannot be
    /// represented by `Q`, or a point-limit error for `kind`.
    pub fn check_container_count(
        &self,
        kind: JsonContainerKind,
        prospective: usize,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        let limit = match kind {
            JsonContainerKind::Sequence => self
                .target
                .limits()
                .structure_limits()
                .sequence_items_limit(),
            JsonContainerKind::Map => {
                self.target.limits().structure_limits().map_entries_limit()
            }
        };
        self.check_container_items(prospective, limit)
    }

    /// Publishes every successful staged admission to the target budget.
    ///
    /// Consumes this transaction. Dropping an uncommitted transaction has no
    /// effect on the target budget.
    pub fn commit(self) {
        self.target.state = self.working;
    }

    /// Returns staged node usage when the cumulative node limit is configured.
    #[must_use]
    pub fn used_nodes(&self) -> Option<Q> {
        self.working.remaining_nodes().map(|remaining| {
            self.target
                .limits()
                .max_nodes()
                .expect("configured nodes limit")
                - remaining
        })
    }

    /// Returns staged remaining node capacity when the node limit is
    /// configured.
    #[must_use]
    pub const fn remaining_nodes(&self) -> Option<Q> {
        self.working.remaining_nodes()
    }

    /// Returns staged payload usage when the payload limit is configured.
    #[must_use]
    pub fn used_payload_bytes(&self) -> Option<Q> {
        self.working.remaining_payload_bytes().map(|remaining| {
            self.target
                .limits()
                .max_payload_bytes()
                .expect("configured payload limit")
                - remaining
        })
    }

    /// Returns staged remaining payload capacity when that limit is configured.
    #[must_use]
    pub const fn remaining_payload_bytes(&self) -> Option<Q> {
        self.working.remaining_payload_bytes()
    }

    /// Checks cumulative capacity for an event without changing working state.
    fn check_cumulative(
        &self,
        prepared: PreparedJsonAdmission<Q>,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        let (node, payload_bytes) = cumulative_cost(prepared);
        if node {
            self.check_nodes()?;
        }
        self.check_payload(payload_bytes)
    }

    /// Converts and checks one prospective container count without mutation.
    fn check_container_items(
        &self,
        amount: usize,
        limit: Option<&crate::ResourceLimit<R, Q>>,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        let Some(limit) = limit else {
            return Ok(());
        };
        let amount = Q::try_from_usize(amount).map_err(|source| {
            MeasuredBudgetError::quantity(limit.resource().clone(), source)
        })?;
        limit.check(amount).map_err(MeasuredBudgetError::from)
    }

    /// Checks the configured node budget for one additional value node.
    fn check_nodes(&self) -> Result<(), MeasuredBudgetError<R, Q>> {
        let Some(remaining) = self.working.remaining_nodes() else {
            return Ok(());
        };
        if Q::ONE <= remaining {
            return Ok(());
        }
        let limit = self
            .target
            .limits()
            .structure_limits()
            .nodes_limit()
            .expect("working node state requires a configured limit");
        Err(InsufficientBudgetError {
            resource: limit.resource().clone(),
            limit: limit.maximum(),
            remaining,
            requested: Q::ONE,
        }
        .into())
    }

    /// Checks the configured payload budget for one event without mutation.
    fn check_payload(
        &self,
        payload_bytes: Q,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        let Some(remaining) = self.working.remaining_payload_bytes() else {
            return Ok(());
        };
        if payload_bytes <= remaining {
            return Ok(());
        }
        let limit = self
            .target
            .limits()
            .payload_bytes_limit()
            .expect("working payload state requires a configured limit");
        Err(InsufficientBudgetError {
            resource: limit.resource().clone(),
            limit: limit.maximum(),
            remaining,
            requested: payload_bytes,
        }
        .into())
    }

    /// Applies a previously checked event to the fixed-size working state.
    fn apply(&mut self, prepared: PreparedJsonAdmission<Q>) {
        let (node, payload_bytes) = cumulative_cost(prepared);
        self.working.apply(node, payload_bytes);
    }
}

/// Returns the cumulative node and payload cost of a prepared JSON event.
fn cumulative_cost<Q>(prepared: PreparedJsonAdmission<Q>) -> (bool, Q)
where
    Q: ResourceQuantity,
{
    match prepared {
        PreparedJsonAdmission::Null { .. }
        | PreparedJsonAdmission::Boolean { .. }
        | PreparedJsonAdmission::Array { .. }
        | PreparedJsonAdmission::Object { .. } => (true, Q::ZERO),
        PreparedJsonAdmission::String { bytes, .. }
        | PreparedJsonAdmission::Number { bytes, .. } => (true, bytes),
        PreparedJsonAdmission::Key { bytes } => (false, bytes),
    }
}
