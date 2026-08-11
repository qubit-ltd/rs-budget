// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors returned by transactional string rendering.

use std::fmt::Debug;
use std::fmt::Display;
use std::string::FromUtf8Error;

use thiserror::Error;

use crate::BudgetError;

/// Describes why a budgeted string rendering transaction failed.
#[derive(Debug, Error)]
pub enum BudgetedStringError<R, E>
where
    R: Debug,
    E: Debug + Display,
{
    /// The rendered prefix exceeded the remaining resource budget.
    #[error(transparent)]
    Budget(BudgetError<R, usize>),
    /// The renderer returned an error unrelated to the budget writer.
    #[error("string renderer failed: {0}")]
    Render(E),
    /// The renderer produced bytes that are not valid UTF-8.
    #[error("rendered bytes are not valid UTF-8")]
    InvalidUtf8(#[source] FromUtf8Error),
    /// The rendered byte length overflowed `usize`.
    #[error("rendered string length overflowed usize")]
    LengthOverflow,
}
