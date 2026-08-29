// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Transactional string output helpers backed by finite resource budgets.

mod budgeted_string_error;
mod budgeted_string_writer;
mod internal;

pub use budgeted_string_error::BudgetedStringError;
pub use budgeted_string_writer::BudgetedStringWriter;
pub(crate) use budgeted_string_writer::render_budgeted_string;
