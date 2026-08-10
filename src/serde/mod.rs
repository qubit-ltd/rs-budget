// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Budget-aware JSON/Serde adapters.

mod internal;
mod json_serde_error;
mod serde_json;

pub use json_serde_error::JsonSerdeError;
pub use serde_json::from_slice_seed_with_budget;
pub use serde_json::from_slice_with_budget;
pub use serde_json::to_vec_with_budget;
pub use serde_json::to_writer_with_budget;
