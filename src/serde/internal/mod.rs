// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private helpers for budget-aware JSON/Serde adapters.

mod json_preflight;
mod json_preflight_child_seed;
mod json_preflight_visitor;

pub(in crate::serde) use json_preflight::JsonPreflight;
pub(in crate::serde) use json_preflight_child_seed::JsonPreflightChildSeed;
pub(in crate::serde) use json_preflight_visitor::JsonPreflightVisitor;
