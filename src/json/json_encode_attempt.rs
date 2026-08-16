// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Provides transactional value accounting for one JSON encode attempt.

use super::JsonMeasurement;
use super::JsonValueTransaction;
use crate::MeasuredBudgetError;
use crate::ResourceBudget;
use crate::ResourceQuantity;

/// I/O accounting and transactional value admission for one JSON encode.
///
/// Dropping an attempt rolls back JSON value accounting. Accepted output
/// charges remain committed, including while unwinding from a panic.
#[must_use = "dropping the attempt rolls back JSON value accounting; accepted output charges remain"]
pub struct JsonEncodeAttempt<'a, R, Q>
where
    Q: ResourceQuantity,
{
    /// Budget charged for accepted output bytes.
    output: Option<&'a mut ResourceBudget<R, Q>>,
    /// Working JSON value accounting published only by [`Self::commit`].
    value: JsonValueTransaction<'a, R, Q>,
}

impl<'a, R, Q> JsonEncodeAttempt<'a, R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Creates an attempt from the budgets split out of an encode session.
    #[inline(always)]
    pub(crate) const fn new(
        output: Option<&'a mut ResourceBudget<R, Q>>,
        value: JsonValueTransaction<'a, R, Q>,
    ) -> Self {
        Self { output, value }
    }

    /// Checks whether output bytes fit without charging them.
    ///
    /// Returns a quantity-conversion or budget error without changing the
    /// configured output budget. An absent output budget is ignored.
    #[inline]
    pub fn check_output_bytes(&self, amount: usize) -> Result<(), MeasuredBudgetError<R, Q>> {
        match self.output.as_deref() {
            Some(budget) => budget.check_available_usize(amount),
            None => Ok(()),
        }
    }

    /// Charges accepted output bytes immediately when the budget is set.
    ///
    /// Returns a quantity-conversion or budget error without changing the
    /// configured output budget on failure. An absent budget is ignored.
    #[inline]
    pub fn try_consume_output_bytes(
        &mut self,
        amount: usize,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        match self.output.as_deref_mut() {
            Some(budget) => budget.try_consume_usize(amount),
            None => Ok(()),
        }
    }

    /// Stages one JSON measurement for publication by [`Self::commit`].
    ///
    /// Returns the transaction's conversion or value-limit error. A failure
    /// leaves this attempt's working value state and output charges unchanged.
    #[inline]
    pub fn try_admit(
        &mut self,
        measurement: JsonMeasurement,
    ) -> Result<(), MeasuredBudgetError<R, Q>> {
        self.value.try_admit(measurement)
    }

    /// Returns the output budget while the attempt exclusively owns it.
    #[must_use = "the output budget reports immediately charged bytes"]
    #[inline(always)]
    pub fn output_budget(&self) -> Option<&ResourceBudget<R, Q>> {
        self.output.as_deref()
    }

    /// Splits this attempt into immediate output and staged value accounting.
    ///
    /// The output budget, when configured, records accepted bytes immediately.
    /// The returned transaction keeps its value changes staged until this
    /// attempt is committed. Dropping the attempt rolls back only that value
    /// state.
    #[must_use = "the returned output budget and value transaction perform the encode accounting"]
    #[inline]
    pub fn split_mut(
        &mut self,
    ) -> (
        Option<&mut ResourceBudget<R, Q>>,
        &mut JsonValueTransaction<'a, R, Q>,
    ) {
        (self.output.as_deref_mut(), &mut self.value)
    }

    /// Returns staged node usage when the node limit is configured.
    #[must_use]
    #[inline]
    pub fn used_nodes(&self) -> Option<Q> {
        self.value.used_nodes()
    }

    /// Returns staged remaining node capacity when the node limit is set.
    #[must_use]
    #[inline(always)]
    pub const fn remaining_nodes(&self) -> Option<Q> {
        self.value.remaining_nodes()
    }

    /// Returns staged payload usage when the payload limit is configured.
    #[must_use]
    #[inline]
    pub fn used_payload_bytes(&self) -> Option<Q> {
        self.value.used_payload_bytes()
    }

    /// Returns staged remaining payload capacity when the payload limit is set.
    #[must_use]
    #[inline(always)]
    pub const fn remaining_payload_bytes(&self) -> Option<Q> {
        self.value.remaining_payload_bytes()
    }

    /// Returns the mutable transaction that holds this attempt's value state.
    #[must_use = "the returned transaction must be used for JSON value admission"]
    #[inline]
    pub fn value_transaction_mut(&mut self) -> &mut JsonValueTransaction<'a, R, Q> {
        &mut self.value
    }

    /// Publishes this attempt's staged value state without rolling back output.
    pub fn commit(self) {
        self.value.commit();
    }
}
