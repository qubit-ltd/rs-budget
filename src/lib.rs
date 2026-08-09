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

mod resource_budget;
mod resource_budget_error;
mod resource_pool;
mod resource_pool_error;
mod resource_quantity;

pub use resource_budget::ResourceBudget;
pub use resource_budget_error::ResourceBudgetError;
pub use resource_pool::ResourcePool;
pub use resource_pool_error::ResourcePoolError;
pub use resource_quantity::ResourceQuantity;

pub mod time;

pub use time::DurationBudget;
pub use time::DurationBudgetError;
#[cfg(feature = "time")]
pub use time::TimeBudget;
#[cfg(feature = "time")]
pub use time::TimeBudgetError;
