// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
// =============================================================================
//! Finite budgets for explicit durations and continuous monotonic deadlines.

mod duration_budget;
mod duration_budget_error;
mod time_budget;
mod time_budget_error;

pub use duration_budget::DurationBudget;
pub use duration_budget_error::DurationBudgetError;
pub use time_budget::TimeBudget;
pub use time_budget_error::TimeBudgetError;
