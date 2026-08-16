// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Core resource limits, budgets, pools, quantities, and accounting errors.

mod budget_error;
mod budget_group_error;
mod insufficient_budget_error;
mod limit_exceeded_error;
mod measured_budget_error;
mod observation;
mod quantity_conversion_error;
mod quantity_measurement;
mod resource_budget;
mod resource_limit;
mod resource_pool;
mod resource_quantity;
mod resource_release_error;

pub use budget_error::BudgetError;
pub use budget_group_error::BudgetGroupError;
pub use insufficient_budget_error::InsufficientBudgetError;
pub use limit_exceeded_error::LimitExceededError;
pub use measured_budget_error::MeasuredBudgetError;
pub use observation::Observation;
pub use quantity_conversion_error::QuantityConversionError;
pub use quantity_measurement::QuantityMeasurement;
pub use resource_budget::ResourceBudget;
pub use resource_limit::ResourceLimit;
pub(crate) use resource_limit::check_limit;
pub use resource_pool::ResourcePool;
pub use resource_quantity::ResourceQuantity;
pub use resource_release_error::ResourceReleaseError;
