// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private formatting adapter for budgeted string rendering.
// qubit-style: allow source-test-pair

use std::fmt;

use crate::resource::ResourceQuantity;
use crate::string::BudgetedStringWriter;

/// Formatting adapter that appends through a budgeted string writer.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
pub(crate) struct FmtWriter<'writer, 'budget, R, Q>
where
    Q: ResourceQuantity,
{
    /// Writer receiving formatted bytes.
    pub(crate) writer: &'writer mut BudgetedStringWriter<'budget, R, Q>,
}

impl<R, Q> fmt::Write for FmtWriter<'_, '_, R, Q>
where
    R: Clone,
    Q: ResourceQuantity,
{
    /// Appends one formatted string when the budget permits it.
    ///
    /// # Errors
    ///
    /// Returns `fmt::Error` when the enclosing budget rejects the append.
    ///
    /// # Parameters
    ///
    /// * `value` - Value to measure or validate.
    ///
    /// # Returns
    ///
    /// Appends one formatted string when the budget permits it.
    fn write_str(&mut self, value: &str) -> fmt::Result {
        if self.writer.append(value.as_bytes()) {
            Ok(())
        } else {
            Err(fmt::Error)
        }
    }
}
