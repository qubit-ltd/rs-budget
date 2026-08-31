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
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `E` - Error type returned by the caller-provided renderer.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Examples
///
/// ```
/// use qubit_budget::BudgetedStringError;
///
/// let error = BudgetedStringError::<&str, &str>::Render("render failed");
/// assert!(matches!(error, BudgetedStringError::Render("render failed")));
/// ```
#[derive(Debug, Error)]
#[must_use]
pub enum BudgetedStringError<R, E, Q = u64>
where
    R: Debug,
    E: Debug + Display,
    Q: Copy + Debug,
{
    /// The rendered prefix exceeded the remaining resource budget.
    #[error(transparent)]
    Budget(
        /// Exact resource, capacity, balance, and rejected request.
        InsufficientBudgetError<R, Q>,
    ),
    /// The output buffer could not reserve the requested capacity.
    #[error("string output allocation failed: {0}")]
    Allocation(
        /// Allocation failure returned by the output byte buffer.
        #[source]
        TryReserveError,
    ),
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
    Render(
        /// Original error returned by the caller-provided renderer.
        E,
    ),
    /// The renderer produced bytes that are not valid UTF-8.
    #[error("rendered bytes are not valid UTF-8")]
    InvalidUtf8(
        /// UTF-8 conversion failure retaining the rendered bytes.
        #[source]
        FromUtf8Error,
    ),
    /// The rendered byte length overflowed `usize`.
    #[error("rendered string length overflowed usize")]
    LengthOverflow,
}
