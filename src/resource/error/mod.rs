// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Errors produced by resource limits, budgets, pools, and conversions.

mod budget_error;
mod budget_group_error;
mod insufficient_budget_error;
mod limit_exceeded_error;
mod measured_budget_error;
mod quantity_conversion_error;
mod resource_release_error;

pub use budget_error::BudgetError;
pub use budget_group_error::BudgetGroupError;
pub use insufficient_budget_error::InsufficientBudgetError;
pub use limit_exceeded_error::LimitExceededError;
pub use measured_budget_error::MeasuredBudgetError;
pub use quantity_conversion_error::QuantityConversionError;
pub use resource_release_error::ResourceReleaseError;
