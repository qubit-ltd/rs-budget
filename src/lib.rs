// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![cfg_attr(docsrs, feature(doc_cfg))]
//! Dependency-light finite resource limits, budgets and pools.
//!
//! A budget object always represents a configured finite constraint. When a
//! dimension is unconfigured, callers use `Option::None` and do not create a
//! no-op or unlimited budget object. Resource quantities use an exact unsigned
//! integer type. High-level limits remain generic so callers can preserve their
//! native measurements; conversions from machine-sized measurements are checked
//! and reported as resource-accounting errors.

mod budget_error;
mod budget_group_error;
mod measured_budget_error;
mod observation;
mod quantity_conversion_error;
mod resource_budget;
mod resource_limit;
mod resource_pool;
mod resource_quantity;
mod resource_release_error;
pub mod string;
mod value;

#[cfg(feature = "json")]
pub mod json;
pub mod structure;

pub use budget_error::BudgetError;
pub use budget_group_error::BudgetGroupError;
pub use measured_budget_error::MeasuredBudgetError;
pub use observation::Observation;
pub use quantity_conversion_error::QuantityConversionError;
pub use quantity_conversion_error::QuantityMeasurement;
pub use resource_budget::ResourceBudget;
pub use resource_limit::ResourceLimit;
pub use resource_pool::ResourcePool;
pub use resource_quantity::ResourceQuantity;
pub use resource_release_error::ResourceReleaseError;
pub use string::BudgetedStringError;
pub use string::BudgetedStringWriter;
pub use structure::StructureBudget;
pub use structure::StructureLimits;
pub use structure::StructureResource;
#[cfg(feature = "big-decimal")]
pub use value::BigDecimalLimits;
#[cfg(feature = "big-integer")]
pub use value::BigIntegerLimits;
pub use value::StringLimits;

pub mod time;

pub use time::DurationBudget;
#[cfg(feature = "time")]
pub use time::TimeBudget;
#[cfg(feature = "time")]
pub use time::TimeBudgetError;
