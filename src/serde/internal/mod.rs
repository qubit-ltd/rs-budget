// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private helpers for budget-aware JSON/Serde adapters.

mod json_budget_compound;
mod json_budget_serializer;
mod json_output_writer;
mod json_preflight;

pub(in crate::serde) use json_budget_serializer::JsonBudgetSerializer;
pub(in crate::serde) use json_output_writer::JsonOutputWriter;
pub(in crate::serde) use json_preflight::JsonPreflight;
