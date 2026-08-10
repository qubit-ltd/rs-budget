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
//! integer type, defaulting to `u64`.

mod budget_error;
mod resource_budget;
mod resource_limit;
mod resource_pool;
mod resource_quantity;
mod structure_budget;
mod structure_limits;
mod structure_resource;

#[cfg(feature = "json")]
pub mod json;

pub use budget_error::BudgetError;
pub use resource_budget::ResourceBudget;
pub use resource_limit::ResourceLimit;
pub use resource_pool::ResourcePool;
pub use resource_quantity::ResourceQuantity;
pub use structure_budget::StructureBudget;
pub use structure_limits::StructureLimits;
pub use structure_resource::StructureResource;

#[cfg(feature = "json")]
pub use json::JsonBudget;
#[cfg(feature = "json")]
pub use json::JsonLimits;
#[cfg(feature = "json")]
pub use json::JsonResource;

pub mod time;

pub use time::DurationBudget;
#[cfg(feature = "time")]
pub use time::TimeBudget;
#[cfg(feature = "time")]
pub use time::TimeBudgetError;
