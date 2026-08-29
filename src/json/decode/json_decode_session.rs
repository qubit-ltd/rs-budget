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
use super::internal::DecodeStorage;
use crate::json::JsonResource;
use crate::json::JsonValueBudget;
use crate::resource::ResourceBudget;
use crate::resource::ResourceQuantity;

/// Mutable resource accounting for one JSON decoding operation.
///
/// Use [`Self::from_limits`] for a session that owns budgets created from immutable
/// limits, or one of the `borrowing_*` constructors when the caller owns the
/// budgets. Create an attempt with [`Self::begin_value`] for each complete
/// value; input charges are immediate, while value accounting is committed by
/// [`JsonDecodeAttempt::commit`].
///
/// # Examples
///
/// ```
/// use qubit_budget::json::JsonDecodeLimits;
/// use qubit_budget::json::JsonDecodeSession;
/// use qubit_budget::json::JsonMeasurement;
///
/// let limits = JsonDecodeLimits::builder()
///     .max_input_bytes(4_usize)
///     .max_nodes(1_usize)
///     .build();
/// let mut session = JsonDecodeSession::from_limits(limits);
/// let mut attempt = session.begin_value();
/// attempt
///     .try_consume_input_bytes(4)
///     .expect("the input should fit");
/// attempt
///     .try_admit(JsonMeasurement::Null { depth: 1 })
///     .expect("the value should fit");
/// attempt.commit();
/// assert_eq!(session.input_budget().expect("input budget").used(), 4);
/// ```
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
    #[inline]
    #[must_use]
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
    #[inline]
    #[must_use]
    pub fn borrowing_input(input: &'a mut ResourceBudget<R, Q>, value: &'a mut JsonValueBudget<R, Q>) -> Self {
        Self {
            storage: DecodeStorage::Borrowed {
                input: Some(input),
                normalized_input: None,
                value,
            },
        }
    }

    /// Creates a session borrowing all caller-owned decode budgets.
    #[inline]
    #[must_use]
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
    #[must_use]
    pub fn begin_value(&mut self) -> JsonDecodeAttempt<'_, R, Q> {
        let (input, normalized_input, value) = self.storage.split();
        JsonDecodeAttempt::new(input, normalized_input, value.transaction())
    }

    /// Returns the raw input budget when configured.
    #[must_use]
    #[inline(always)]
    pub fn input_budget(&self) -> Option<&ResourceBudget<R, Q>> {
        match &self.storage {
            DecodeStorage::Owned { input, .. } => input.as_ref(),
            DecodeStorage::Borrowed { input, .. } => input.as_deref(),
        }
    }

    /// Returns the configured raw input-byte maximum.
    #[must_use]
    #[inline(always)]
    pub fn max_input_bytes(&self) -> Option<Q> {
        self.input_budget().map(ResourceBudget::limit)
    }

    /// Returns the configured normalized input-byte maximum.
    #[must_use]
    #[inline(always)]
    pub fn max_normalized_input_bytes(&self) -> Option<Q> {
        self.normalized_input_budget().map(ResourceBudget::limit)
    }

    /// Returns the normalized input budget when configured.
    #[must_use]
    #[inline(always)]
    pub fn normalized_input_budget(&self) -> Option<&ResourceBudget<R, Q>> {
        match &self.storage {
            DecodeStorage::Owned { normalized_input, .. } => normalized_input.as_ref(),
            DecodeStorage::Borrowed { normalized_input, .. } => normalized_input.as_deref(),
        }
    }

    /// Returns the value budget for read-only inspection.
    #[must_use]
    #[inline(always)]
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
    /// Creates a session that owns budgets initialized from immutable limits.
    #[inline]
    #[must_use]
    pub fn from_limits(limits: JsonDecodeLimits<R, Q>) -> Self {
        let input = limits.input_bytes_limit().cloned().map(ResourceBudget::from_limit);
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
