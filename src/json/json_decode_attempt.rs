// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Provides transactional value accounting for one JSON decode attempt.

use super::JsonMeasurement;
use super::JsonValueTransaction;
use crate::MeasuredBudgetError;
use crate::ResourceBudget;
use crate::ResourceQuantity;

/// I/O accounting and transactional value admission for one JSON decode.
///
/// Dropping an attempt rolls back JSON value accounting. Raw and normalized
/// input charges remain committed, including while unwinding from a panic.
#[must_use = "dropping the attempt rolls back JSON value accounting; input charges remain"]
pub struct JsonDecodeAttempt<'a, R, Q>
where
    Q: ResourceQuantity,
{
    /// Budget charged for raw input bytes.
    input: Option<&'a mut ResourceBudget<R, Q>>,
    /// Budget charged for normalized input bytes.
    normalized_input: Option<&'a mut ResourceBudget<R, Q>>,
    /// Working JSON value accounting published only by [`Self::commit`].
    value: JsonValueTransaction<'a, R, Q>,
}

impl<'a, R, Q> JsonDecodeAttempt<'a, R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates an attempt from the budgets split out of a decode session.
    pub(crate) const fn new(
        input: Option<&'a mut ResourceBudget<R, Q>>,
        normalized_input: Option<&'a mut ResourceBudget<R, Q>>,
        value: JsonValueTransaction<'a, R, Q>,
    ) -> Self {
        Self {
            input,
            normalized_input,
            value,
        }
    }

    /// Charges raw input bytes immediately when that budget is configured.
    ///
    /// Returns a quantity-conversion or budget error without changing the
    /// configured input budget on failure. An absent input budget is ignored.
    pub fn try_consume_input_bytes(
        &mut self,
        amount: usize,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        consume_bytes(self.input.as_deref_mut(), amount)
    }

    /// Charges normalized input bytes immediately when that budget is set.
    ///
    /// Returns a quantity-conversion or budget error without changing the
    /// configured normalized budget on failure. An absent budget is ignored.
    pub fn try_consume_normalized_input_bytes(
        &mut self,
        amount: usize,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        consume_bytes(self.normalized_input.as_deref_mut(), amount)
    }

    /// Stages one JSON measurement for publication by [`Self::commit`].
    ///
    /// Returns the transaction's conversion or value-limit error. A failure
    /// leaves this attempt's working value state and all I/O charges unchanged.
    pub fn try_admit(
        &mut self,
        measurement: JsonMeasurement,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        self.value.try_admit(measurement)
    }

    /// Returns the raw input budget while the attempt exclusively owns it.
    #[must_use = "the raw input budget reports immediately charged bytes"]
    pub fn input_budget(&self) -> Option<&ResourceBudget<R, Q>> {
        self.input.as_deref()
    }

    /// Returns the normalized input budget while the attempt owns it.
    #[must_use = "the normalized budget reports immediately charged bytes"]
    pub fn normalized_input_budget(&self) -> Option<&ResourceBudget<R, Q>> {
        self.normalized_input.as_deref()
    }

    /// Returns staged node usage when the node limit is configured.
    #[must_use]
    pub fn used_nodes(&self) -> Option<Q> {
        self.value.used_nodes()
    }

    /// Returns staged remaining node capacity when the node limit is set.
    #[must_use]
    pub const fn remaining_nodes(&self) -> Option<Q> {
        self.value.remaining_nodes()
    }

    /// Returns staged payload usage when the payload limit is configured.
    #[must_use]
    pub fn used_payload_bytes(&self) -> Option<Q> {
        self.value.used_payload_bytes()
    }

    /// Returns staged remaining payload capacity when the payload limit is set.
    #[must_use]
    pub const fn remaining_payload_bytes(&self) -> Option<Q> {
        self.value.remaining_payload_bytes()
    }

    /// Returns the mutable transaction that holds this attempt's value state.
    #[must_use = "the returned transaction must be used for JSON value admission"]
    pub fn value_transaction_mut(
        &mut self,
    ) -> &mut JsonValueTransaction<'a, R, Q> {
        &mut self.value
    }

    /// Publishes this attempt's staged value state without rolling back I/O.
    pub fn commit(self) {
        self.value.commit();
    }
}

/// Converts and immediately consumes native bytes when a budget is present.
fn consume_bytes<R, Q>(
    budget: Option<&mut ResourceBudget<R, Q>>,
    amount: usize,
) -> Result<(), MeasuredBudgetError<R, Q>>
where
    R: Clone,
    Q: ResourceQuantity,
{
    match budget {
        Some(budget) => budget.try_consume_usize(amount),
        None => Ok(()),
    }
}
