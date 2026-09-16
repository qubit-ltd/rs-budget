// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Adds transactional UTF-8 output to [`ResourceBudget`].

use std::fmt;

use super::ResourceBudget;
use crate::resource::ResourceQuantity;
use crate::string::BudgetedStringError;
use crate::string::BudgetedStringWriter;
use crate::string::render_budgeted_string;

impl<R, Q> ResourceBudget<R, Q>
where
    R: Clone + fmt::Debug,
    Q: ResourceQuantity,
{
    /// Renders and transactionally commits a UTF-8 string under this budget.
    ///
    /// # Type Parameters
    ///
    /// * `E` - Error type returned by the caller-provided renderer.
    /// * `F` - Closure used to render the output string.
    ///
    /// # Parameters
    ///
    /// * `render` - Caller-provided renderer writing into the transactional
    ///   adapter.
    ///
    /// # Returns
    ///
    /// `Ok(rendered)` after the complete UTF-8 output is charged and committed.
    ///
    /// # Errors
    ///
    /// Returns [`BudgetedStringError`] when rendering, allocation, UTF-8
    /// validation, measurement, or budget accounting fails.
    #[inline(always)]
    pub fn try_write_string<E, F>(&mut self, render: F) -> Result<String, BudgetedStringError<R, E, Q>>
    where
        E: fmt::Debug + fmt::Display,
        F: FnOnce(&mut BudgetedStringWriter<'_, R, Q>) -> Result<(), E>,
    {
        render_budgeted_string(self, render)
    }
}
