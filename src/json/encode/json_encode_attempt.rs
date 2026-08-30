// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Provides transactional value accounting for one JSON encode attempt.

use crate::json::JsonMeasurement;
use crate::json::JsonValueTransaction;
use crate::resource::MeasuredBudgetError;
use crate::resource::ResourceBudget;
use crate::resource::ResourceQuantity;

/// I/O accounting and transactional value admission for one JSON encode.
///
/// Dropping an attempt rolls back JSON value accounting. Accepted output
/// charges remain committed, including while unwinding from a panic.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Examples
///
/// ```
/// use qubit_budget::json::JsonEncodeLimits;
/// use qubit_budget::json::JsonEncodeSession;
/// use qubit_budget::json::JsonMeasurement;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let limits = JsonEncodeLimits::builder().max_nodes(1_usize).build();
/// let mut session = JsonEncodeSession::from_limits(limits);
/// let mut attempt = session.begin_value();
/// attempt.try_admit(JsonMeasurement::Null { depth: 1 }).expect("null should fit");
/// attempt.commit()?;
/// # Ok(()) }
/// ```
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
    ///
    /// # Parameters
    ///
    /// * `output` - Output supplied to this operation.
    /// * `value` - Transaction holding the JSON value accounting staged by this
    ///   attempt.
    ///
    /// # Returns
    ///
    /// Creates an attempt from the budgets split out of an encode session.
    #[inline(always)]
    #[must_use]
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
    ///
    /// # Parameters
    ///
    /// * `amount` - Number of output bytes to check without charging.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the operation completes successfully.
    ///
    /// # Errors
    ///
    /// Returns [`MeasuredBudgetError`] when a native measurement cannot fit `Q`
    /// or a configured limit rejects it.
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
    ///
    /// # Parameters
    ///
    /// * `amount` - Number of accepted output bytes to charge immediately.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the operation completes successfully.
    ///
    /// # Errors
    ///
    /// Returns [`MeasuredBudgetError`] when a native measurement cannot fit `Q`
    /// or a configured limit rejects it.
    #[inline]
    pub fn try_consume_output_bytes(&mut self, amount: usize) -> Result<(), MeasuredBudgetError<R, Q>> {
        match self.output.as_deref_mut() {
            Some(budget) => budget.try_consume_usize(amount),
            None => Ok(()),
        }
    }

    /// Stages one JSON measurement for publication by [`Self::commit`].
    ///
    /// Returns the transaction's conversion or value-limit error. A failure
    /// leaves this attempt's working value state and output charges unchanged,
    /// and poisons later value admissions and commit. Output failures alone do
    /// not poison the value transaction.
    ///
    /// # Parameters
    ///
    /// * `measurement` - Native JSON measurement to convert or admit.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the operation completes successfully.
    ///
    /// # Errors
    ///
    /// Returns [`MeasuredBudgetError`] when a native measurement cannot fit `Q`
    /// or a configured limit rejects it.
    #[inline]
    pub fn try_admit(&mut self, measurement: JsonMeasurement) -> Result<(), MeasuredBudgetError<R, Q>> {
        self.value.try_admit(measurement)
    }

    /// Returns the output budget while the attempt exclusively owns it.
    ///
    /// # Returns
    ///
    /// Returns the output budget while the attempt exclusively owns it.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
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
    ///
    /// # Returns
    ///
    /// Splits this attempt into immediate output and staged value accounting.
    ///
    /// A `None` output budget indicates that output-byte accounting is
    /// unconfigured.
    #[must_use]
    #[inline]
    pub fn split_mut(&mut self) -> (Option<&mut ResourceBudget<R, Q>>, &mut JsonValueTransaction<'a, R, Q>) {
        (self.output.as_deref_mut(), &mut self.value)
    }

    /// Returns staged node usage when the node limit is configured.
    ///
    /// # Returns
    ///
    /// Returns staged node usage when the node limit is configured.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline]
    pub fn used_nodes(&self) -> Option<Q> {
        self.value.used_nodes()
    }

    /// Returns staged remaining node capacity when the node limit is set.
    ///
    /// # Returns
    ///
    /// Returns staged remaining node capacity when the node limit is set.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub const fn remaining_nodes(&self) -> Option<Q> {
        self.value.remaining_nodes()
    }

    /// Returns staged payload usage when the payload limit is configured.
    ///
    /// # Returns
    ///
    /// Returns staged payload usage when the payload limit is configured.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline]
    pub fn used_payload_bytes(&self) -> Option<Q> {
        self.value.used_payload_bytes()
    }

    /// Returns staged remaining payload capacity when the payload limit is set.
    ///
    /// # Returns
    ///
    /// Returns staged remaining payload capacity when the payload limit is set.
    ///
    /// `None` indicates that the corresponding limit or budget dimension is
    /// unconfigured.
    #[must_use]
    #[inline(always)]
    pub const fn remaining_payload_bytes(&self) -> Option<Q> {
        self.value.remaining_payload_bytes()
    }

    /// Returns the mutable transaction that holds this attempt's value state.
    ///
    /// # Returns
    ///
    /// Returns the mutable transaction that holds this attempt's value state.
    #[must_use]
    #[inline]
    pub fn value_transaction_mut(&mut self) -> &mut JsonValueTransaction<'a, R, Q> {
        &mut self.value
    }

    /// Publishes this attempt's staged value state without rolling back output.
    ///
    /// # Returns
    ///
    /// `Ok(())` after publishing the staged value state.
    ///
    /// # Errors
    ///
    /// Returns the first value-admission error when the attempt is poisoned.
    pub fn commit(self) -> Result<(), MeasuredBudgetError<R, Q>> {
        self.value.commit()
    }
}
