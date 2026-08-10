// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Resource limits and budgets for JSON processing without a JSON parser.

mod json_budget;
mod json_limits;
mod json_resource;

pub use json_budget::JsonBudget;
pub use json_limits::JsonLimits;
pub use json_resource::JsonResource;
