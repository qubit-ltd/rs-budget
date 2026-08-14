// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private failures captured while rendering a budgeted string.

use crate::MeasuredBudgetError;
use crate::ResourceQuantity;

/// Failure state retained until the string transaction is finalized.
pub(crate) enum WriterFailure<R, Q>
where
    Q: ResourceQuantity,
{
    /// A resource budget or quantity conversion rejected output.
    Budget(MeasuredBudgetError<R, Q>),
    /// The buffered output length overflowed `usize`.
    LengthOverflow,
}
