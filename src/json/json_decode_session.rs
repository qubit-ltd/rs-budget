// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tracks mutable accounting for one JSON decoding operation.

use crate::BudgetError;
use crate::JsonDecodeLimits;
use crate::JsonResource;
use crate::JsonValueBudget;
use crate::ResourceBudget;
use crate::ResourceQuantity;

/// Mutable state for one JSON decoding operation.
///
/// Input bytes and JSON value resources are intentionally separate: only input
/// bytes are directional, while the embedded [`JsonValueBudget`] accounts for
/// decoded JSON values.
#[must_use]
#[derive(Debug, PartialEq, Eq)]
pub struct JsonDecodeSession<R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    /// Optional cumulative input-byte accounting.
    input: Option<ResourceBudget<R, Q>>,

    /// Direction-independent JSON value accounting.
    value: JsonValueBudget<R, Q>,
}

impl<R, Q> JsonDecodeSession<R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates fresh mutable accounting for one JSON decoding operation.
    #[inline]
    pub fn new(limits: JsonDecodeLimits<R, Q>) -> Self {
        let input = limits
            .input_bytes_limit()
            .cloned()
            .map(ResourceBudget::from_limit);
        let value = JsonValueBudget::new(limits.value_limits());
        Self { input, value }
    }

    /// Consumes input bytes atomically for this decoding operation.
    ///
    /// A failed request leaves the remaining input capacity unchanged.
    #[inline]
    pub fn consume_input_bytes(
        &mut self,
        amount: Q,
    ) -> Result<(), BudgetError<R, Q>> {
        match &mut self.input {
            Some(input) => input.try_consume(amount),
            None => Ok(()),
        }
    }

    /// Returns the configured cumulative input-byte maximum.
    #[must_use]
    #[inline(always)]
    pub const fn max_input_bytes(&self) -> Option<Q> {
        match self.input.as_ref() {
            Some(input) => Some(input.limit()),
            None => None,
        }
    }

    /// Returns the input-byte budget, when input accounting is configured.
    #[must_use = "the input budget tracks the total bytes consumed by this decode operation"]
    #[inline(always)]
    pub const fn input_budget(&self) -> Option<&ResourceBudget<R, Q>> {
        self.input.as_ref()
    }

    /// Returns the JSON value budget for read-only inspection.
    #[must_use = "the value budget tracks decoded JSON nodes, structure and payload"]
    #[inline(always)]
    pub const fn value_budget(&self) -> &JsonValueBudget<R, Q> {
        &self.value
    }

    /// Returns the JSON value budget for mutable JSON traversal accounting.
    #[inline(always)]
    pub fn value_budget_mut(&mut self) -> &mut JsonValueBudget<R, Q> {
        &mut self.value
    }
}
