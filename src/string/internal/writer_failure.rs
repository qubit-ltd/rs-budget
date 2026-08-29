// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private failures captured while rendering a budgeted string.
// qubit-style: allow source-test-pair

use std::collections::TryReserveError;

use crate::resource::InsufficientBudgetError;
use crate::resource::QuantityConversionError;
use crate::resource::ResourceQuantity;

/// Failure state retained until the string transaction is finalized.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
pub(crate) enum WriterFailure<R, Q>
where
    Q: ResourceQuantity,
{
    /// The output exceeded the remaining budget.
    Budget(
        /// Exact resource, capacity, balance, and rejected request.
        InsufficientBudgetError<R, Q>,
    ),
    /// The output length could not be represented by the budget quantity.
    Quantity {
        /// Resource whose accounting required the conversion.
        resource: R,
        /// Failed quantity conversion.
        source: QuantityConversionError,
    },
    /// The output buffer could not reserve the requested capacity.
    Allocation(
        /// Allocation failure returned by the output byte buffer.
        TryReserveError,
    ),
    /// The buffered output length overflowed `usize`.
    LengthOverflow,
}
