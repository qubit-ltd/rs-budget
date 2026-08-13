// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tracks mutable accounting for one JSON decoding operation.

use super::JsonDecodeLimits;
use super::JsonResource;
use super::JsonValueBudget;
use crate::BudgetError;
use crate::MeasuredBudgetError;
use crate::ResourceBudget;
use crate::ResourceQuantity;

/// Backing storage for owned and caller-borrowed decode budgets.
#[derive(Debug)]
enum DecodeStorage<'a, R, Q>
where
    Q: ResourceQuantity,
{
    Owned {
        input: Option<ResourceBudget<R, Q>>,
        normalized_input: Option<ResourceBudget<R, Q>>,
        value: JsonValueBudget<R, Q>,
    },
    Borrowed {
        input: Option<&'a mut ResourceBudget<R, Q>>,
        normalized_input: Option<&'a mut ResourceBudget<R, Q>>,
        value: &'a mut JsonValueBudget<R, Q>,
    },
}

/// Mutable resource accounting for one JSON decoding operation.
#[must_use]
#[derive(Debug)]
pub struct JsonDecodeSession<'a, R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    storage: DecodeStorage<'a, R, Q>,
}

impl<'a, R, Q> JsonDecodeSession<'a, R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates a session borrowing caller-owned input and value budgets.
    pub fn borrowing(
        input: Option<&'a mut ResourceBudget<R, Q>>,
        normalized_input: Option<&'a mut ResourceBudget<R, Q>>,
        value: &'a mut JsonValueBudget<R, Q>,
    ) -> Self {
        Self {
            storage: DecodeStorage::Borrowed {
                input,
                normalized_input,
                value,
            },
        }
    }

    /// Consumes raw input bytes.
    pub fn consume_input_bytes(
        &mut self,
        amount: Q,
    ) -> Result<(), BudgetError<R, Q>> {
        self.input_budget_mut()
            .map_or(Ok(()), |budget| budget.try_consume(amount))
    }

    /// Converts and consumes raw input bytes.
    pub fn consume_input_bytes_usize(
        &mut self,
        amount: usize,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        consume_usize(self.input_budget_mut(), amount)
    }

    /// Consumes normalized input bytes.
    pub fn consume_normalized_input_bytes(
        &mut self,
        amount: Q,
    ) -> Result<(), BudgetError<R, Q>> {
        self.normalized_input_budget_mut()
            .map_or(Ok(()), |budget| budget.try_consume(amount))
    }

    /// Converts and consumes normalized input bytes.
    pub fn consume_normalized_input_bytes_usize(
        &mut self,
        amount: usize,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        consume_usize(self.normalized_input_budget_mut(), amount)
    }

    /// Returns the raw input budget when configured.
    pub fn input_budget(&self) -> Option<&ResourceBudget<R, Q>> {
        match &self.storage {
            DecodeStorage::Owned { input, .. } => input.as_ref(),
            DecodeStorage::Borrowed { input, .. } => input.as_deref(),
        }
    }

    /// Returns the configured raw input-byte maximum.
    pub fn max_input_bytes(&self) -> Option<Q> {
        self.input_budget().map(ResourceBudget::limit)
    }

    /// Returns the configured normalized input-byte maximum.
    pub fn max_normalized_input_bytes(&self) -> Option<Q> {
        self.normalized_input_budget().map(ResourceBudget::limit)
    }

    /// Returns the normalized input budget when configured.
    pub fn normalized_input_budget(&self) -> Option<&ResourceBudget<R, Q>> {
        match &self.storage {
            DecodeStorage::Owned {
                normalized_input, ..
            } => normalized_input.as_ref(),
            DecodeStorage::Borrowed {
                normalized_input, ..
            } => normalized_input.as_deref(),
        }
    }

    /// Returns the value budget for read-only inspection.
    pub fn value_budget(&self) -> &JsonValueBudget<R, Q> {
        match &self.storage {
            DecodeStorage::Owned { value, .. } => value,
            DecodeStorage::Borrowed { value, .. } => value,
        }
    }

    /// Returns the value budget for traversal accounting.
    pub fn value_budget_mut(&mut self) -> &mut JsonValueBudget<R, Q> {
        match &mut self.storage {
            DecodeStorage::Owned { value, .. } => value,
            DecodeStorage::Borrowed { value, .. } => value,
        }
    }

    /// Returns a mutable raw input budget when configured.
    fn input_budget_mut(&mut self) -> Option<&mut ResourceBudget<R, Q>> {
        match &mut self.storage {
            DecodeStorage::Owned { input, .. } => input.as_mut(),
            DecodeStorage::Borrowed { input, .. } => input.as_deref_mut(),
        }
    }

    /// Returns a mutable normalized-input budget when configured.
    fn normalized_input_budget_mut(
        &mut self,
    ) -> Option<&mut ResourceBudget<R, Q>> {
        match &mut self.storage {
            DecodeStorage::Owned {
                normalized_input, ..
            } => normalized_input.as_mut(),
            DecodeStorage::Borrowed {
                normalized_input, ..
            } => normalized_input.as_deref_mut(),
        }
    }
}

impl<R, Q> JsonDecodeSession<'static, R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates an owned session from immutable limits.
    pub fn owned(limits: JsonDecodeLimits<R, Q>) -> Self {
        let input = limits
            .input_bytes_limit()
            .cloned()
            .map(ResourceBudget::from_limit);
        let normalized_input = limits
            .normalized_input_bytes_limit()
            .cloned()
            .map(ResourceBudget::from_limit);
        let value = JsonValueBudget::new(limits.value_limits());
        Self {
            storage: DecodeStorage::Owned {
                input,
                normalized_input,
                value,
            },
        }
    }
}

/// Converts and consumes a machine-sized measurement when a budget exists.
fn consume_usize<R, Q>(
    budget: Option<&mut ResourceBudget<R, Q>>,
    amount: usize,
) -> Result<(), MeasuredBudgetError<R, Q>>
where
    R: Clone,
    Q: ResourceQuantity,
{
    let Some(budget) = budget else {
        return Ok(());
    };
    let amount = Q::try_from_usize(amount).map_err(|source| {
        MeasuredBudgetError::quantity(budget.resource().clone(), source)
    })?;
    budget
        .try_consume(amount)
        .map_err(MeasuredBudgetError::from)
}
