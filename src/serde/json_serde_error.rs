// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines errors returned by the budget-aware JSON/Serde adapters.

use std::fmt;

use serde_json::Error as JsonError;
use thiserror::Error;

use crate::BudgetError;

/// Errors returned by budget-aware JSON/Serde adapters.
#[must_use]
#[derive(Debug, Error)]
pub enum JsonSerdeError<R, Q = usize>
where
    Q: Copy + fmt::Debug,
{
    /// The document exceeded one configured resource budget.
    #[error("JSON resource budget exceeded: {0}")]
    Budget(#[source] BudgetError<R, Q>),

    /// Serde JSON rejected the document or value.
    #[error("JSON serialization error: {0}")]
    Json(#[source] JsonError),

    /// The destination writer rejected serialized bytes.
    #[error("JSON output writer failed: {0}")]
    Io(#[source] std::io::Error),
}
