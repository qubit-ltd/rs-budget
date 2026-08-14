// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private I/O adapter for budgeted string rendering.

use std::io;

use crate::ResourceQuantity;
use crate::string::BudgetedStringWriter;

/// I/O adapter that appends through a budgeted string writer.
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
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
