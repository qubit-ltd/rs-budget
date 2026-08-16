// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Immutable point limits and their observations.

mod observation;
mod resource_limit;

pub use observation::Observation;
pub use resource_limit::ResourceLimit;
pub(crate) use resource_limit::check_limit;
