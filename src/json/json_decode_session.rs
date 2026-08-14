// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tracks mutable accounting for one JSON decoding operation.

use super::JsonDecodeAttempt;
use super::JsonDecodeLimits;
use super::JsonResource;
use super::JsonValueBudget;
use super::internal::DecodeStorage;
use crate::ResourceBudget;
use crate::ResourceQuantity;

/// Mutable resource accounting for one JSON decoding operation.
#[must_use]
#[derive(Debug)]
pub struct JsonDecodeSession<'a, R = JsonResource, Q = usize>
where
    Q: ResourceQuantity,
{
    /// Owned or borrowed budgets backing this decode operation.
    storage: DecodeStorage<'a, R, Q>,
}

impl<'a, R, Q> JsonDecodeSession<'a, R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates a session borrowing only a caller-owned value budget.
    pub fn borrowing_value(value: &'a mut JsonValueBudget<R, Q>) -> Self {
        Self {
            storage: DecodeStorage::Borrowed {
                input: None,
                normalized_input: None,
                value,
            },
        }
    }

    /// Creates a session borrowing caller-owned raw-input and value budgets.
    pub fn borrowing_input(
        input: &'a mut ResourceBudget<R, Q>,
        value: &'a mut JsonValueBudget<R, Q>,
    ) -> Self {
        Self {
            storage: DecodeStorage::Borrowed {
                input: Some(input),
                normalized_input: None,
                value,
            },
        }
    }

    /// Creates a session borrowing all caller-owned decode budgets.
    pub fn borrowing_all(
        input: &'a mut ResourceBudget<R, Q>,
        normalized_input: &'a mut ResourceBudget<R, Q>,
        value: &'a mut JsonValueBudget<R, Q>,
    ) -> Self {
        Self {
            storage: DecodeStorage::Borrowed {
                input: Some(input),
                normalized_input: Some(normalized_input),
                value,
            },
        }
    }

    /// Starts accounting for one complete JSON value.
    ///
    /// The returned attempt charges raw and normalized input immediately, but
    /// publishes staged JSON value accounting only after `commit`. Dropping it
    /// rolls back only the staged value state.
    #[must_use = "dropping the attempt rolls back JSON value accounting"]
    pub fn begin_value(&mut self) -> JsonDecodeAttempt<'_, R, Q> {
        let (input, normalized_input, value) = self.storage.split();
        JsonDecodeAttempt::new(input, normalized_input, value.transaction())
    }

    /// Returns the raw input budget when configured.
    #[must_use = "the raw input budget reports consumed input bytes"]
    pub fn input_budget(&self) -> Option<&ResourceBudget<R, Q>> {
        match &self.storage {
            DecodeStorage::Owned { input, .. } => input.as_ref(),
            DecodeStorage::Borrowed { input, .. } => input.as_deref(),
        }
    }

    /// Returns the configured raw input-byte maximum.
    #[must_use]
    pub fn max_input_bytes(&self) -> Option<Q> {
        self.input_budget().map(ResourceBudget::limit)
    }

    /// Returns the configured normalized input-byte maximum.
    #[must_use]
    pub fn max_normalized_input_bytes(&self) -> Option<Q> {
        self.normalized_input_budget().map(ResourceBudget::limit)
    }

    /// Returns the normalized input budget when configured.
    #[must_use = "the normalized budget reports consumed normalized bytes"]
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
    #[must_use = "the value budget reports accepted JSON traversal"]
    pub fn value_budget(&self) -> &JsonValueBudget<R, Q> {
        match &self.storage {
            DecodeStorage::Owned { value, .. } => value,
            DecodeStorage::Borrowed { value, .. } => value,
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
        let value = JsonValueBudget::new(limits.into_value_limits());
        Self {
            storage: DecodeStorage::Owned {
                input,
                normalized_input,
                value,
            },
        }
    }
}
