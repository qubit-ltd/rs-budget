// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! JSON value limits and transactional accounting.

mod internal;
mod json_value_budget;
mod json_value_limits;
mod json_value_limits_builder;
mod json_value_transaction;

pub use json_value_budget::JsonValueBudget;
pub use json_value_limits::JsonValueLimits;
pub use json_value_limits_builder::JsonValueLimitsBuilder;
pub use json_value_transaction::JsonValueTransaction;
