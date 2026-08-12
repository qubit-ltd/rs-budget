// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tracks mutable accounting for one JSON encoding operation.

use crate::BudgetError;
use crate::JsonEncodeLimits;
use crate::JsonResource;
use crate::JsonValueBudget;
use crate::ResourceBudget;
use crate::ResourceQuantity;

/// Mutable state for one JSON encoding operation.
///
/// Output bytes and JSON value resources are intentionally separate: only
/// output bytes are directional, while the embedded [`JsonValueBudget`]
/// accounts for encoded JSON values.
#[must_use]
#[derive(Debug, PartialEq, Eq)]
pub struct JsonEncodeSession<R = JsonResource, Q = u64>
where
    Q: ResourceQuantity,
{
    /// Optional cumulative output-byte accounting.
    output: Option<ResourceBudget<R, Q>>,

    /// Direction-independent JSON value accounting.
    value: JsonValueBudget<R, Q>,
}

impl<R, Q> JsonEncodeSession<R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates fresh mutable accounting for one JSON encoding operation.
    #[inline]
    pub fn owned(limits: JsonEncodeLimits<R, Q>) -> Self {
        let output = limits
            .output_bytes_limit()
            .cloned()
            .map(ResourceBudget::from_limit);
        let value = JsonValueBudget::new(limits.value_limits());
        Self { output, value }
    }

    /// Consumes output bytes atomically for this encoding operation.
    ///
    /// A failed request leaves the remaining output capacity unchanged.
    #[inline]
    pub fn consume_output_bytes(
        &mut self,
        amount: Q,
    ) -> Result<(), BudgetError<R, Q>> {
        match &mut self.output {
            Some(output) => output.try_consume(amount),
            None => Ok(()),
        }
    }

    /// Returns the configured cumulative output-byte maximum.
    #[must_use]
    #[inline(always)]
    pub const fn max_output_bytes(&self) -> Option<Q> {
        match self.output.as_ref() {
            Some(output) => Some(output.limit()),
            None => None,
        }
    }

    /// Returns the output-byte budget, when output accounting is configured.
    #[must_use = "the output budget tracks bytes emitted by this encode operation"]
    #[inline(always)]
    pub const fn output_budget(&self) -> Option<&ResourceBudget<R, Q>> {
        self.output.as_ref()
    }

    /// Returns the JSON value budget for read-only inspection.
    #[must_use = "the value budget tracks encoded JSON nodes, structure and payload"]
    #[inline(always)]
    pub const fn value_budget(&self) -> &JsonValueBudget<R, Q> {
        &self.value
    }

    /// Returns the JSON value budget for mutable JSON traversal accounting.
    #[inline(always)]
    pub fn value_budget_mut(&mut self) -> &mut JsonValueBudget<R, Q> {
        &mut self.value
    }

    /// Splits mutable output and value accounting for one online traversal.
    ///
    /// This crate-private operation lets the Serde adapter enforce value
    /// limits before delegation while the output writer independently charges
    /// bytes as they are emitted.
    #[cfg(feature = "serde-json")]
    #[inline(always)]
    pub(crate) fn split_mut(
        &mut self,
    ) -> (
        Option<&mut ResourceBudget<R, Q>>,
        &mut JsonValueBudget<R, Q>,
    ) {
        (self.output.as_mut(), &mut self.value)
    }
}
