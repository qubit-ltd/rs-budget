// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tracks mutable accounting for one JSON encoding operation.

use super::JsonEncodeAttempt;
use super::JsonEncodeLimits;
use super::JsonResource;
use super::JsonValueBudget;
use super::internal::EncodeStorage;
use crate::ResourceBudget;
use crate::ResourceQuantity;

/// Mutable resource accounting for one JSON encoding operation.
#[must_use]
#[derive(Debug)]
pub struct JsonEncodeSession<'a, R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    /// Owned or borrowed budgets backing this encode operation.
    storage: EncodeStorage<'a, R, Q>,
}

impl<'a, R, Q> JsonEncodeSession<'a, R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates a session borrowing only a caller-owned value budget.
    #[inline]
    pub fn borrowing_value(value: &'a mut JsonValueBudget<R, Q>) -> Self {
        Self {
            storage: EncodeStorage::Borrowed {
                output: None,
                value,
            },
        }
    }

    /// Creates a session borrowing caller-owned output and value budgets.
    #[inline]
    pub fn borrowing_output(
        output: &'a mut ResourceBudget<R, Q>,
        value: &'a mut JsonValueBudget<R, Q>,
    ) -> Self {
        Self {
            storage: EncodeStorage::Borrowed {
                output: Some(output),
                value,
            },
        }
    }

    /// Starts accounting for one complete JSON value.
    ///
    /// The returned attempt charges accepted output immediately, but publishes
    /// staged JSON value accounting only after `commit`. Dropping it rolls back
    /// only the staged value state.
    #[must_use = "dropping the attempt rolls back JSON value accounting"]
    pub fn begin_value(&mut self) -> JsonEncodeAttempt<'_, R, Q> {
        let (output, value) = self.storage.split();
        JsonEncodeAttempt::new(output, value.transaction())
    }

    /// Returns the output budget when configured.
    #[must_use = "the output budget reports consumed output bytes"]
    #[inline(always)]
    pub fn output_budget(&self) -> Option<&ResourceBudget<R, Q>> {
        match &self.storage {
            EncodeStorage::Owned { output, .. } => output.as_ref(),
            EncodeStorage::Borrowed { output, .. } => output.as_deref(),
        }
    }

    /// Returns the configured output-byte maximum.
    #[must_use]
    #[inline(always)]
    pub fn max_output_bytes(&self) -> Option<Q> {
        self.output_budget().map(ResourceBudget::limit)
    }

    /// Returns the value budget for read-only inspection.
    #[must_use = "the value budget reports accepted JSON traversal"]
    #[inline(always)]
    pub fn value_budget(&self) -> &JsonValueBudget<R, Q> {
        match &self.storage {
            EncodeStorage::Owned { value, .. } => value,
            EncodeStorage::Borrowed { value, .. } => value,
        }
    }
}

impl<R, Q> JsonEncodeSession<'static, R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates an owned session from immutable limits.
    #[inline]
    pub fn owned(limits: JsonEncodeLimits<R, Q>) -> Self {
        let output = limits
            .output_bytes_limit()
            .cloned()
            .map(ResourceBudget::from_limit);
        let value = JsonValueBudget::new(limits.into_value_limits());
        Self {
            storage: EncodeStorage::Owned { output, value },
        }
    }
}
