// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Dependency-light resource limit and budget accounting primitives.

mod invalid_release;
mod limit_exceeded;
mod resource_budget;
mod resource_limit;

pub use invalid_release::InvalidRelease;
pub use limit_exceeded::LimitExceeded;
pub use resource_budget::ResourceBudget;
pub use resource_limit::ResourceLimit;
