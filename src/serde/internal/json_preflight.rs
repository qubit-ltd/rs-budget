// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Preflight walker that charges JSON structure against a budget.

use serde::de::DeserializeSeed;
use serde::de::Deserializer;
use serde::de::Error as DeError;

use super::JsonPreflightVisitor;
use crate::BudgetError;
use crate::JsonBudget;

/// Charges one JSON document against a mutable budget session.
///
/// # Type Parameters
///
/// * `R` - Resource identity reported by [`BudgetError`] values.
pub(in crate::serde) struct JsonPreflight<'a, R> {
    /// Budget session mutated while walking the document.
    budget: &'a mut JsonBudget<R, usize>,

    /// First budget violation captured during the walk, if any.
    violation: Option<BudgetError<R, usize>>,
}

impl<'a, R> JsonPreflight<'a, R>
where
    R: Clone,
{
    /// Creates a preflight walker bound to one budget session.
    ///
    /// # Parameters
    ///
    /// * `budget` - Mutable JSON budget charged by the walk.
    ///
    /// # Returns
    ///
    /// An empty walker with no recorded violation.
    pub(in crate::serde) fn new(budget: &'a mut JsonBudget<R, usize>) -> Self {
        Self {
            budget,
            violation: None,
        }
    }

    /// Takes the first recorded budget violation, if any.
    ///
    /// # Returns
    ///
    /// `Some` when a budget check failed during the walk; otherwise `None`.
    #[must_use]
    pub(in crate::serde) fn take_violation(
        &mut self,
    ) -> Option<BudgetError<R, usize>> {
        self.violation.take()
    }

    /// Records a budget-check result and maps failure into a Serde error.
    ///
    /// # Parameters
    ///
    /// * `result` - Outcome of one budget charge or limit check.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the budget accepted the charge.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original [`BudgetError`].
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    fn record<E>(
        &mut self,
        result: Result<(), BudgetError<R, usize>>,
    ) -> Result<(), E>
    where
        E: DeError,
    {
        result.map_err(|error| {
            self.violation = Some(error);
            E::custom("JSON resource budget exceeded")
        })
    }

    /// Charges one structural node at the given nesting depth.
    ///
    /// # Parameters
    ///
    /// * `depth` - Inclusive nesting depth of the node being entered.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the depth and node budgets accept the charge.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original [`BudgetError`].
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    pub(in crate::serde) fn node<E>(&mut self, depth: usize) -> Result<(), E>
    where
        E: DeError,
    {
        let result = self.budget.enter_node(depth);
        self.record(result)
    }

    /// Charges the item count of one sequence.
    ///
    /// # Parameters
    ///
    /// * `items` - Inclusive item count observed so far in the sequence.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the sequence-item budget accepts the count.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original [`BudgetError`].
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    pub(in crate::serde) fn sequence_items<E>(
        &mut self,
        items: usize,
    ) -> Result<(), E>
    where
        E: DeError,
    {
        self.record(self.budget.check_sequence_items(items))
    }

    /// Charges the entry count of one map.
    ///
    /// # Parameters
    ///
    /// * `entries` - Inclusive entry count observed so far in the map.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the map-entry budget accepts the count.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original [`BudgetError`].
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    pub(in crate::serde) fn map_entries<E>(
        &mut self,
        entries: usize,
    ) -> Result<(), E>
    where
        E: DeError,
    {
        self.record(self.budget.check_map_entries(entries))
    }

    /// Charges the byte length of one structural key.
    ///
    /// # Parameters
    ///
    /// * `bytes` - UTF-8 byte length of the key.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the key-byte budget accepts the length.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original [`BudgetError`].
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    pub(in crate::serde) fn key<E>(&mut self, bytes: usize) -> Result<(), E>
    where
        E: DeError,
    {
        self.record(self.budget.check_key_bytes(bytes))
    }

    /// Charges the byte length of one JSON string.
    ///
    /// # Parameters
    ///
    /// * `bytes` - UTF-8 byte length of the string value.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the string-byte budget accepts the length.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original [`BudgetError`].
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    pub(in crate::serde) fn string<E>(&mut self, bytes: usize) -> Result<(), E>
    where
        E: DeError,
    {
        self.record(self.budget.check_string_bytes(bytes))
    }

    /// Charges the byte length of one JSON number representation.
    ///
    /// # Parameters
    ///
    /// * `bytes` - Byte length of the number's textual representation.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the number-byte budget accepts the length.
    ///
    /// # Errors
    ///
    /// Returns a Serde error after storing the original [`BudgetError`].
    ///
    /// # Type Parameters
    ///
    /// * `E` - Serde deserializer error type used to surface the failure.
    pub(in crate::serde) fn number<E>(&mut self, bytes: usize) -> Result<(), E>
    where
        E: DeError,
    {
        self.record(self.budget.check_number_bytes(bytes))
    }
}

impl<'de, 'a, 'b, R> DeserializeSeed<'de> for &'a mut JsonPreflight<'b, R>
where
    R: Clone,
{
    type Value = ();

    /// Walks one JSON value and charges it against the bound budget.
    ///
    /// # Parameters
    ///
    /// * `deserializer` - Serde deserializer positioned at the next value.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the value was walked without a Serde failure.
    ///
    /// # Errors
    ///
    /// Returns the deserializer error produced by Serde or by a recorded
    /// budget violation mapped through [`JsonPreflight::record`].
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(JsonPreflightVisitor {
            preflight: self,
            depth: 1,
        })
    }
}
