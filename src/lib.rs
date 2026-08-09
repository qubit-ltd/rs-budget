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
//! no-op or unlimited budget object. All resource quantities use `u64`.

mod limit_exceeded;
mod resource_budget;
mod resource_budget_error;
mod resource_limit;
mod resource_pool;
mod resource_pool_error;

pub use limit_exceeded::LimitExceeded;
pub use resource_budget::ResourceBudget;
pub use resource_budget_error::ResourceBudgetError;
pub use resource_limit::ResourceLimit;
pub use resource_pool::ResourcePool;
pub use resource_pool_error::ResourcePoolError;

#[cfg(feature = "time")]
#[cfg_attr(docsrs, doc(cfg(feature = "time")))]
pub mod time;

#[cfg(feature = "time")]
pub use time::DurationBudget;
#[cfg(feature = "time")]
pub use time::DurationBudgetError;
#[cfg(feature = "time")]
pub use time::TimeBudget;
#[cfg(feature = "time")]
pub use time::TimeBudgetError;
