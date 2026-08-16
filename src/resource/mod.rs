// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Core resource limits, budgets, pools, quantities, and accounting errors.

mod budget;
mod budget_error;
mod budget_group_error;
mod limit;
mod measured_budget_error;
mod observation;
mod pool;
mod quantity;
mod quantity_conversion_error;
mod quantity_measurement;
mod release_error;

pub use budget::ResourceBudget;
pub use budget_error::BudgetError;
pub use budget_error::InsufficientBudgetError;
pub use budget_error::LimitExceededError;
pub use budget_group_error::BudgetGroupError;
pub use limit::ResourceLimit;
pub(crate) use limit::check_limit;
pub use measured_budget_error::MeasuredBudgetError;
pub use observation::Observation;
pub use pool::ResourcePool;
pub use quantity::ResourceQuantity;
pub use quantity_conversion_error::QuantityConversionError;
pub use quantity_measurement::QuantityMeasurement;
pub use release_error::ResourceReleaseError;
