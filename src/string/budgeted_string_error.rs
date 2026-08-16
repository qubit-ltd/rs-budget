// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors returned by transactional string rendering.

use std::collections::TryReserveError;
use std::fmt::Debug;
use std::fmt::Display;
use std::string::FromUtf8Error;

use thiserror::Error;

use crate::resource::InsufficientBudgetError;
use crate::resource::QuantityConversionError;

/// Describes why a budgeted string rendering transaction failed.
#[derive(Debug, Error)]
pub enum BudgetedStringError<R, E, Q = u64>
where
    R: Debug,
    E: Debug + Display,
    Q: Copy + Debug,
{
    /// The rendered prefix exceeded the remaining resource budget.
    #[error(transparent)]
    Budget(InsufficientBudgetError<R, Q>),
    /// The output buffer could not reserve the requested capacity.
    #[error("string output allocation failed: {0}")]
    Allocation(#[source] TryReserveError),
    /// The rendered UTF-8 byte length cannot be represented by the budget
    /// quantity.
    #[error("string byte measurement cannot be represented: {source}")]
    Quantity {
        /// Resource whose accounting required the conversion.
        resource: R,
        /// Failed quantity conversion.
        #[source]
        source: QuantityConversionError,
    },
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
