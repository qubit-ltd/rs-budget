// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Resource limits and budgets for nested structural data.

mod structure_budget;
mod structure_limits;
mod structure_limits_builder;
mod structure_resource;

pub use structure_budget::StructureBudget;
pub use structure_limits::StructureLimits;
pub use structure_limits_builder::StructureLimitsBuilder;
pub use structure_resource::StructureResource;
