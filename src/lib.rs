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
//!
//! A failed single-budget operation is non-mutating, and
//! [`ResourceBudget::try_consume_group`] checks every member before charging
//! any member. Higher-level sessions may deliberately retain charges for work
//! already attempted. Generic [`StructureBudget`] measurements are consumed
//! immediately, while JSON value measurements are staged and commit only after
//! the complete value succeeds. JSON raw and normalized input bytes, plus
//! accepted output prefixes, are charged immediately. These are separate
//! guarantees; an I/O failure does not itself poison a JSON value transaction,
//! and output transactionality does not imply whole-operation rollback.

mod resource;
pub mod string;
pub mod structure;
pub mod time;
mod value;

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
pub mod json;
pub use resource::BudgetError;
pub use resource::BudgetGroupError;
pub use resource::InsufficientBudgetError;
pub use resource::LimitExceededError;
pub use resource::ManagedResourcePermit;
pub use resource::ManagedResourcePool;
pub use resource::MeasuredBudgetError;
pub use resource::Observation;
pub use resource::QuantityConversionError;
pub use resource::QuantityMeasurement;
pub use resource::ResourceBudget;
pub use resource::ResourceLimit;
pub use resource::ResourcePool;
pub use resource::ResourceQuantity;
pub use resource::ResourceReleaseError;
pub use string::BudgetedStringError;
pub use string::BudgetedStringWriter;
pub use structure::StructureBudget;
pub use structure::StructureLimits;
pub use structure::StructureLimitsBuilder;
pub use structure::StructureResource;
pub use time::DurationBudget;
#[cfg(feature = "time")]
#[cfg_attr(docsrs, doc(cfg(feature = "time")))]
pub use time::TimeBudget;
#[cfg(feature = "time")]
#[cfg_attr(docsrs, doc(cfg(feature = "time")))]
pub use time::TimeBudgetError;
#[cfg(feature = "big-decimal")]
#[cfg_attr(docsrs, doc(cfg(feature = "big-decimal")))]
pub use value::BigDecimalLimits;
#[cfg(feature = "big-decimal")]
#[cfg_attr(docsrs, doc(cfg(feature = "big-decimal")))]
pub use value::BigDecimalLimitsBuilder;
#[cfg(feature = "big-integer")]
#[cfg_attr(docsrs, doc(cfg(feature = "big-integer")))]
pub use value::BigIntegerLimits;
#[cfg(feature = "big-integer")]
#[cfg_attr(docsrs, doc(cfg(feature = "big-integer")))]
pub use value::BigIntegerLimitsBuilder;
pub use value::StringLimits;
pub use value::StringLimitsBuilder;
