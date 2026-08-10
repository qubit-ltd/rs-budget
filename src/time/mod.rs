// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Finite budgets for explicit durations and continuous monotonic deadlines.

mod duration_budget;
#[cfg(feature = "time")]
mod time_budget;
#[cfg(feature = "time")]
mod time_budget_error;

pub use duration_budget::DurationBudget;
#[cfg(feature = "time")]
pub use time_budget::TimeBudget;
#[cfg(feature = "time")]
pub use time_budget_error::TimeBudgetError;
