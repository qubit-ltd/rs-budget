// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Resource limits and budgets for JSON processing without a JSON parser.

mod json_decode_limits;
mod json_decode_session;
mod json_encode_limits;
mod json_encode_session;
mod json_resource;
mod json_value_budget;
mod json_value_limits;

pub use json_decode_limits::JsonDecodeLimits;
pub use json_decode_session::JsonDecodeSession;
pub use json_encode_limits::JsonEncodeLimits;
pub use json_encode_session::JsonEncodeSession;
pub use json_resource::JsonResource;
pub use json_value_budget::JsonValueBudget;
pub use json_value_limits::JsonValueLimits;
