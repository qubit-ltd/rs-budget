// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Bounded output buffer for budget-aware JSON serialization.

use std::io;
use std::io::Write;

use crate::BudgetError;
use crate::JsonBudget;
use crate::JsonSerdeError;

/// Accumulates JSON bytes only while the configured output budget permits it.
pub(in crate::serde) struct JsonOutputWriter<'a, R> {
    /// Bytes accepted by the output budget so far.
    bytes: Vec<u8>,

    /// Independent budget session used for output-byte point checks.
    budget: &'a JsonBudget<R, usize>,

    /// Original budget violation hidden behind an `io::Error`, if any.
    violation: Option<BudgetError<R, usize>>,
}

impl<'a, R> JsonOutputWriter<'a, R> {
    /// Creates an empty bounded output buffer.
    ///
    /// # Parameters
    ///
    /// * `budget` - Budget whose output-byte limit guards buffer growth.
    ///
    /// # Returns
    ///
    /// An empty writer with no recorded violation.
    #[inline]
    pub(in crate::serde) const fn new(budget: &'a JsonBudget<R, usize>) -> Self {
        Self {
            bytes: Vec::new(),
            budget,
            violation: None,
        }
    }

    /// Returns the number of bytes currently retained by the test buffer.
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn buffered_len(&self) -> usize {
        self.bytes.len()
    }

    /// Resolves serialization and returns the accepted bytes or original error.
    ///
    /// # Parameters
    ///
    /// * `result` - Result returned by the JSON serializer.
    ///
    /// # Returns
    ///
    /// The complete bounded output when serialization succeeds.
    ///
    /// # Errors
    ///
    /// Returns [`JsonSerdeError::Budget`] for a recorded output violation,
    /// taking precedence over its erased I/O representation. Otherwise returns
    /// [`JsonSerdeError::Json`] for the serializer failure.
    pub(in crate::serde) fn into_result(
        self,
        result: Result<(), serde_json::Error>,
    ) -> Result<Vec<u8>, JsonSerdeError<R>> {
        if let Some(error) = self.violation {
            return Err(JsonSerdeError::Budget(error));
        }
        result.map_err(JsonSerdeError::Json)?;
        Ok(self.bytes)
    }
}

impl<R> Write for JsonOutputWriter<'_, R>
where
    R: Clone,
{
    /// Appends one complete input slice after checking the resulting length.
    ///
    /// The buffer remains unchanged if arithmetic overflows or the output-byte
    /// limit is exceeded.
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        let next = self
            .bytes
            .len()
            .checked_add(input.len())
            .ok_or_else(|| io::Error::other("JSON output length overflow"))?;
        if let Err(error) = self.budget.check_output_bytes(next) {
            self.violation = Some(error);
            return Err(io::Error::other("JSON output budget exceeded"));
        }
        self.bytes.extend_from_slice(input);
        Ok(input.len())
    }

    /// Flushes the in-memory buffer without performing external I/O.
    #[inline(always)]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/serde/internal/json_output_writer_tests.rs"]
mod json_output_writer_tests;
