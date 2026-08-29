// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private I/O adapter for budgeted string rendering.
// qubit-style: allow source-test-pair

use std::io;

use crate::resource::ResourceQuantity;
use crate::string::BudgetedStringWriter;

/// I/O adapter that appends through a budgeted string writer.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
pub(crate) struct IoWriter<'writer, 'budget, R, Q>
where
    Q: ResourceQuantity,
{
    /// Writer receiving output bytes.
    pub(crate) writer: &'writer mut BudgetedStringWriter<'budget, R, Q>,
}

impl<R, Q> io::Write for IoWriter<'_, '_, R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Appends bytes when the budget permits them.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when the enclosing budget rejects the append.
    ///
    /// # Parameters
    ///
    /// * `bytes` - Native byte count to validate or charge.
    ///
    /// # Returns
    ///
    /// Appends bytes when the budget permits them.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.writer.append(bytes) {
            Ok(bytes.len())
        } else {
            Err(io::Error::other("budgeted string writer rejected output"))
        }
    }

    /// Flushes the in-memory adapter.
    ///
    /// # Errors
    ///
    /// This in-memory adapter never fails while flushing.
    ///
    /// # Returns
    ///
    /// Flushes the in-memory adapter.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
