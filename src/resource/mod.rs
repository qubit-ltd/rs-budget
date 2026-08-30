// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Core resource limits, budgets, pools, quantities, and accounting errors.

mod budget;
mod error;
mod limit;
mod quantity;

pub use budget::ManagedResourcePermit;
pub use budget::ManagedResourcePool;
pub use budget::ResourceBudget;
pub use budget::ResourcePool;
pub use error::BudgetError;
pub use error::BudgetGroupError;
pub use error::InsufficientBudgetError;
pub use error::LimitExceededError;
pub use error::MeasuredBudgetError;
pub use error::QuantityConversionError;
pub use error::ResourceReleaseError;
pub use limit::Observation;
pub use limit::ResourceLimit;
pub(crate) use limit::check_limit;
pub use quantity::QuantityMeasurement;
pub use quantity::ResourceQuantity;
