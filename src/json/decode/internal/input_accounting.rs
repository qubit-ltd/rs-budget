// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private immediate accounting for JSON decoder input bytes.

use crate::resource::MeasuredBudgetError;
use crate::resource::ResourceBudget;
use crate::resource::ResourceQuantity;

/// Converts and immediately consumes native bytes when a budget is present.
///
/// # Type Parameters
///
/// * `R` - Caller-defined resource identity retained by limits and errors.
/// * `Q` - Exact unsigned quantity used for measurements and accounting.
///
/// # Parameters
///
/// * `budget` - Optional cumulative byte budget to charge.
/// * `amount` - Native byte count to convert and charge when `budget` exists.
///
/// # Returns
///
/// `Ok(())` when the operation completes successfully.
///
/// # Errors
///
/// Returns [`MeasuredBudgetError`] when `amount` cannot fit `Q` or exceeds the
/// budget's remaining capacity.
#[inline]
pub(in crate::json::decode) fn consume_bytes<R, Q>(
    budget: Option<&mut ResourceBudget<R, Q>>,
    amount: usize,
) -> Result<(), MeasuredBudgetError<R, Q>>
where
    R: Clone,
    Q: ResourceQuantity,
{
    match budget {
        Some(budget) => budget.try_consume_usize(amount),
        None => Ok(()),
    }
}
