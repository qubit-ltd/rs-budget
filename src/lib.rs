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
//! integer type. Generic accounting remains parameterized, while the default
//! structural limits use `usize` for collection-sized measurements. String,
//! big-number, and JSON value limits use `u64` for stable cross-target byte,
//! count, and depth semantics.

mod budget_error;
mod observation;
mod resource_budget;
mod resource_limit;
mod resource_pool;
mod resource_quantity;
mod resource_release_error;
pub mod string;
mod value;

pub mod structure;

#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "serde-json")]
pub mod serde;

pub use budget_error::BudgetError;
#[cfg(feature = "json")]
pub use json::JsonDecodeLimits;
#[cfg(feature = "json")]
pub use json::JsonDecodeSession;
#[cfg(feature = "json")]
pub use json::JsonEncodeLimits;
#[cfg(feature = "json")]
pub use json::JsonEncodeSession;
#[cfg(feature = "json")]
pub use json::JsonResource;
#[cfg(feature = "json")]
pub use json::JsonValueBudget;
#[cfg(feature = "json")]
pub use json::JsonValueLimits;
pub use observation::Observation;
pub use resource_budget::ResourceBudget;
pub use resource_limit::ResourceLimit;
pub use resource_pool::ResourcePool;
pub use resource_quantity::ResourceQuantity;
pub use resource_release_error::ResourceReleaseError;
#[cfg(feature = "serde-json")]
pub use serde::JsonSerdeError;
#[cfg(feature = "serde-json")]
pub use serde::decode_slice;
#[cfg(feature = "serde-json")]
pub use serde::decode_slice_seed;
#[cfg(feature = "serde-json")]
pub use serde::encode_to_vec;
#[cfg(feature = "serde-json")]
pub use serde::encode_to_writer;
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
