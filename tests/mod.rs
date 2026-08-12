// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
mod budget_error_tests;
mod budget_group_error_tests;
mod observation_tests;
mod resource_budget_tests;
mod resource_limit_tests;
mod resource_pool_tests;
mod resource_quantity_tests;
mod resource_release_error_tests;
mod string;
mod value;

mod structure;

#[cfg(feature = "json")]
mod json;

#[cfg(feature = "serde-json")]
mod serde;

mod time;
