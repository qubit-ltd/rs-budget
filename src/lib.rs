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
//! already attempted: JSON input bytes and accepted structural traversal are
//! consumed as they are admitted. JSON value accounting is transactional and
//! commits only after the complete value succeeds. JSON output charges accepted
//! prefixes immediately; buffered output commits only after the complete
//! document succeeds. These are separate guarantees; output transactionality
//! does not imply whole-operation rollback for input or structure accounting.

mod internal;
mod resource;
pub mod string;
mod value;

#[cfg(feature = "json")]
pub mod json;
pub mod structure;

pub use resource::BudgetError;
pub use resource::BudgetGroupError;
pub use resource::InsufficientBudgetError;
pub use resource::LimitExceededError;
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
#[cfg(feature = "big-decimal")]
pub use value::BigDecimalLimits;
#[cfg(feature = "big-decimal")]
pub use value::BigDecimalLimitsBuilder;
#[cfg(feature = "big-integer")]
pub use value::BigIntegerLimits;
#[cfg(feature = "big-integer")]
pub use value::BigIntegerLimitsBuilder;
pub use value::StringLimits;
pub use value::StringLimitsBuilder;

pub mod time;

pub use time::DurationBudget;
#[cfg(feature = "time")]
pub use time::TimeBudget;
#[cfg(feature = "time")]
pub use time::TimeBudgetError;
