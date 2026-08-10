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
//! integer type. Generic resource budgets default to `u64`; the structural and
//! JSON limit families default to `usize` because their measurements are
//! normally byte lengths or collection sizes.

mod budget_error;
mod resource_budget;
mod resource_limit;
mod resource_pool;
mod resource_quantity;
mod resource_release_error;
mod structure_budget;
mod structure_limits;
mod structure_resource;

#[cfg(feature = "serde-json")]
mod json_serde_error;

#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "serde-json")]
pub mod serde_json;

pub use budget_error::BudgetError;
#[cfg(feature = "json")]
pub use json::JsonBudget;
#[cfg(feature = "json")]
pub use json::JsonLimits;
#[cfg(feature = "json")]
pub use json::JsonResource;
#[cfg(feature = "serde-json")]
pub use json_serde_error::JsonSerdeError;
pub use resource_budget::ResourceBudget;
pub use resource_limit::ResourceLimit;
pub use resource_pool::ResourcePool;
pub use resource_quantity::ResourceQuantity;
pub use resource_release_error::ResourceReleaseError;
#[cfg(feature = "serde-json")]
pub use serde_json::from_slice_seed_with_budget;
#[cfg(feature = "serde-json")]
pub use serde_json::from_slice_with_budget;
#[cfg(feature = "serde-json")]
pub use serde_json::to_vec_with_budget;
#[cfg(feature = "serde-json")]
pub use serde_json::to_writer_with_budget;
pub use structure_budget::StructureBudget;
pub use structure_limits::StructureLimits;
pub use structure_resource::StructureResource;

pub mod time;

pub use time::DurationBudget;
#[cfg(feature = "time")]
pub use time::TimeBudget;
#[cfg(feature = "time")]
pub use time::TimeBudgetError;
