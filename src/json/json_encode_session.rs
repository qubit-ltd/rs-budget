// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tracks mutable accounting for one JSON encoding operation.

use crate::BudgetError;
use crate::MeasuredBudgetError;
use crate::ResourceBudget;
use crate::ResourceQuantity;

use super::JsonEncodeLimits;
use super::JsonResource;
use super::JsonValueBudget;

/// Backing storage for owned and caller-borrowed encode budgets.
#[derive(Debug)]
enum EncodeStorage<'a, R, Q>
where
    Q: ResourceQuantity,
{
    Owned {
        output: Option<ResourceBudget<R, Q>>,
        value: JsonValueBudget<R, Q>,
    },
    Borrowed {
        output: Option<&'a mut ResourceBudget<R, Q>>,
        value: &'a mut JsonValueBudget<R, Q>,
    },
}

/// Mutable resource accounting for one JSON encoding operation.
#[must_use]
#[derive(Debug)]
pub struct JsonEncodeSession<'a, R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    storage: EncodeStorage<'a, R, Q>,
}

impl<'a, R, Q> JsonEncodeSession<'a, R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates a session borrowing caller-owned output and value budgets.
    pub fn borrowing(
        output: Option<&'a mut ResourceBudget<R, Q>>,
        value: &'a mut JsonValueBudget<R, Q>,
    ) -> Self {
        Self {
            storage: EncodeStorage::Borrowed { output, value },
        }
    }

    /// Consumes output bytes.
    pub fn consume_output_bytes(&mut self, amount: Q) -> Result<(), BudgetError<R, Q>> {
        self.output_budget_mut()
            .map_or(Ok(()), |budget| budget.try_consume(amount))
    }

    /// Converts and consumes output bytes.
    pub fn consume_output_bytes_usize(
        &mut self,
        amount: usize,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        let Some(budget) = self.output_budget_mut() else {
            return Ok(());
        };
        let amount = Q::try_from_usize(amount)
            .map_err(|source| MeasuredBudgetError::quantity(budget.resource().clone(), source))?;
        budget
            .try_consume(amount)
            .map_err(MeasuredBudgetError::from)
    }

    /// Returns the output budget when configured.
    pub fn output_budget(&self) -> Option<&ResourceBudget<R, Q>> {
        match &self.storage {
            EncodeStorage::Owned { output, .. } => output.as_ref(),
            EncodeStorage::Borrowed { output, .. } => output.as_deref(),
        }
    }

    /// Returns the configured output-byte maximum.
    pub fn max_output_bytes(&self) -> Option<Q> {
        self.output_budget().map(ResourceBudget::limit)
    }

    /// Returns the value budget for read-only inspection.
    pub fn value_budget(&self) -> &JsonValueBudget<R, Q> {
        match &self.storage {
            EncodeStorage::Owned { value, .. } => value,
            EncodeStorage::Borrowed { value, .. } => value,
        }
    }

    /// Returns the value budget for traversal accounting.
    pub fn value_budget_mut(&mut self) -> &mut JsonValueBudget<R, Q> {
        match &mut self.storage {
            EncodeStorage::Owned { value, .. } => value,
            EncodeStorage::Borrowed { value, .. } => value,
        }
    }

    /// Returns a mutable output budget when configured.
    fn output_budget_mut(&mut self) -> Option<&mut ResourceBudget<R, Q>> {
        match &mut self.storage {
            EncodeStorage::Owned { output, .. } => output.as_mut(),
            EncodeStorage::Borrowed { output, .. } => output.as_deref_mut(),
        }
    }
}

impl<R, Q> JsonEncodeSession<'static, R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates an owned session from immutable limits.
    pub fn owned(limits: JsonEncodeLimits<R, Q>) -> Self {
        let output = limits
            .output_bytes_limit()
            .cloned()
            .map(ResourceBudget::from_limit);
        let value = JsonValueBudget::new(limits.value_limits());
        Self {
            storage: EncodeStorage::Owned { output, value },
        }
    }
}
